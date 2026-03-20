//! Procedural macros for Ash workflow language
//!
//! This crate provides derive macros for common Ash patterns:
//! - `Effectful`: Automatically track effect levels for workflow nodes
//! - `Provenance`: Generate provenance recording code
//! - `Trace`: Derive trace event generation

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/// Derive macro for types that participate in effect tracking
///
/// Generates an implementation of `Effectful` trait that computes
/// the minimal effect level required to execute this workflow node.
///
/// # Example
/// ```ignore
/// use ash_core::effect::{Effect, Effectful};
/// use ash_macros::Effectful;
///
/// #[derive(Effectful)]
/// #[effect(Epistemic)]
/// struct ObserveWorkflow {
///     capability: String,
/// }
///
/// // Generated impl:
/// // impl Effectful for ObserveWorkflow {
/// //     fn effect(&self) -> Effect { Effect::Epistemic }
/// // }
/// ```
#[proc_macro_derive(Effectful, attributes(effect))]
pub fn derive_effectful(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    // Look for #[effect(...)] attribute to determine effect level
    let effect_level = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("effect"))
        .and_then(|attr| attr.parse_args::<syn::Ident>().ok())
        .map(|ident| ident.to_string())
        .unwrap_or_else(|| "Epistemic".to_string());

    let effect_variant = syn::Ident::new(&effect_level, proc_macro2::Span::call_site());

    let expanded = quote! {
        impl #impl_generics ::ash_core::effect::Effectful for #name #type_generics #where_clause {
            fn effect(&self) -> ::ash_core::effect::Effect {
                ::ash_core::effect::Effect::#effect_variant
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for types that support provenance tracking
///
/// Generates code to record provenance information during execution.
///
/// # Example
/// ```ignore
/// use ash_macros::Provenance;
///
/// #[derive(Provenance)]
/// struct ActionResult {
///     value: i64,
/// }
/// ```
#[proc_macro_derive(Provenance, attributes(provenance))]
pub fn derive_provenance(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics ::ash_core::provenance::Prov for #name #type_generics #where_clause {
            fn provenance(&self) -> ::ash_core::provenance::Provenance {
                ::ash_core::provenance::Provenance::derived_from(self)
            }
        }
    };

    TokenStream::from(expanded)
}

/// Attribute macro for workflow entry points
///
/// Marks a function as a workflow entry point and generates
/// boilerplate for effect tracking and error handling.
///
/// # Example
/// ```ignore
/// use ash_macros::workflow;
///
/// #[workflow]
/// fn process_data(input: String) -> Result<Value, Error> {
///     // Workflow implementation
/// }
/// ```
#[proc_macro_attribute]
pub fn workflow(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemFn);

    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;
    let attrs = &input.attrs;
    let fn_name = sig.ident.to_string();

    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            let __workflow_id = ::ash_core::WorkflowId::new();
            let __trace_recorder = ::ash_provenance::create_trace_recorder(__workflow_id);
            let __trace_session = ::ash_provenance::WorkflowTraceSession::start(
                __trace_recorder,
                #fn_name,
            ).expect("workflow trace session starts");

            // Execute the workflow body
            let __result = (|| #block)();

            let __trace_recorder = match &__result {
                Ok(_) => __trace_session.finish_success(),
                Err(error) => __trace_session.finish_error(
                    format!("{:?}", error),
                    Some(#fn_name),
                ),
            }.expect("workflow trace session completes");
            let _ = __trace_recorder;

            __result
        }
    };

    TokenStream::from(expanded)
}
