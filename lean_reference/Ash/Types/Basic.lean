/- # Ash Type System - Basic Definitions

Core type definitions for the Ash type system.
Per SPEC-004 Section 4 (Type System) and Section 5 (Semantics).

This module defines:
- The `Ty` type representing Ash types
- Type constructors for primitives, products, sums, and functions
-/ 

import Ash.Core.AST

namespace Ash.Types

open Ash

/-! ## Types

The `Ty` inductive type represents all types in the Ash language.

Per SPEC-004 Section 4.1:
- Base types: Int, String, Bool
- Product types: tuples
- Sum types: variants (enums)
- Function types: arrows (not fully supported in eval)
-/ 
inductive Ty where
  | int : Ty
  | string : Ty
  | bool : Ty
  | tuple : List Ty → Ty
  | variant : String → List (String × Ty) → Ty
  | record : List (String × Ty) → Ty
  | arrow : Ty → Ty → Ty  -- T1 → T2
  | unit : Ty
  deriving BEq, Repr

/-! ## Type Operations

Helper functions for working with types.
-/ 

namespace Ty

/-- Check if a type is a base type (Int, String, Bool, Unit) -/
def isBase : Ty → Bool
  | int => true
  | string => true
  | bool => true
  | unit => true
  | _ => false

/-- Check if a type is a product type (tuple) -/
def isProduct : Ty → Bool
  | tuple _ => true
  | _ => false

/-- Check if a type is a sum type (variant) -/
def isSum : Ty → Bool
  | variant _ _ => true
  | _ => false

/-- Get the arity of a product type -/
def arity : Ty → Nat
  | tuple tys => tys.length
  | _ => 0

end Ty

/-! ## ToString Instance -/

mutual
partial def tyToString : Ty → String
  | Ty.int => "Int"
  | Ty.string => "String"
  | Ty.bool => "Bool"
  | Ty.unit => "Unit"
  | Ty.tuple tys =>
      let inner := String.intercalate ", " (tys.map tyToString)
      "(" ++ inner ++ ")"
  | Ty.variant name fields =>
      let fieldStrs := fields.map (fun (n, t) => n ++ ": " ++ tyToString t)
      let inner := String.intercalate ", " fieldStrs
      name ++ " { " ++ inner ++ " }"
  | Ty.record fields =>
      let fieldStrs := fields.map (fun (n, t) => n ++ ": " ++ tyToString t)
      let inner := String.intercalate ", " fieldStrs
      "{ " ++ inner ++ " }"
  | Ty.arrow t1 t2 =>
      "(" ++ tyToString t1 ++ " → " ++ tyToString t2 ++ ")"
end

instance : ToString Ty where
  toString := tyToString

/-! ## Value Has Type Relation

Runtime values carry their type information.
This relation connects runtime values to static types.

Note: Defined here to avoid circular dependencies between proof modules.
Per SPEC-004 Section 4: Values have types based on their constructors.
-/ 
inductive ValueHasType : Value → Ty → Prop where
  | int {n} : ValueHasType (.int n) .int
  | string {s} : ValueHasType (.string s) .string
  | bool {b} : ValueHasType (.bool b) .bool
  | null : ValueHasType .null .unit
  -- TODO: Add cases for tuple, variant, record (see long-term tasks)

end Ash.Types
