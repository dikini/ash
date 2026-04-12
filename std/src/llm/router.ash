-- Model Router Workflow (SPEC-029 §8.3)
--
-- Routes requests to different models based on content classification.
-- Uses a classifier model to determine which target model is best suited
-- for the request, then dispatches to that model.
--
-- This enables dynamic model selection based on task complexity, cost
-- optimization, or specialized capabilities.
--
-- Example:
--   -- Route coding questions to Claude, general questions to GPT-4o-mini
--   let targets = [
--     ("claude", "claude-sonnet-4"),
--     ("gpt", "gpt-4o-mini")
--   ];
--   let response = router("openai", "gpt-4o", targets, history, "Write a Python function...");

use types::{Message, ChatResponse};
use prompt::{user, append};
use dispatch::complete;

-- Route classification result
-- Indicates which target model should handle the request
type RouteTarget = Coding | General | Complex | Simple | Creative | Factual;

-- Classify the user message to determine routing
--
-- Parameters:
--   provider: Provider name for the classifier model
--   classifier_model: Model to use for classification
--   history: Conversation history
--   user_message: The message to classify
--
-- Returns: RouteTarget indicating which model type to use
fn classify_route(
    provider: String,
    classifier_model: String,
    history: List<Message>,
    user_message: String
) -> RouteTarget {
    -- Build classification prompt
    let classify_prompt = string::concat(
        "Classify the following user request into one category: "
        "CODING (code generation/debugging), "
        "COMPLEX (complex reasoning), "
        "CREATIVE (creative writing), "
        "FACTUAL (factual lookup), "
        "SIMPLE (simple Q&A), or "
        "GENERAL (general conversation).\n\n"
        "Respond with ONLY the category name in uppercase.\n\n"
        "User request: ",
        user_message
    );

    let classify_msg = user(classify_prompt);
    let classify_history = append(history, classify_msg);

    let response = complete(provider, classifier_model, classify_history, None);

    match response.content {
        None => General,
        Some { value: text } => {
            let upper = string::to_uppercase(text);
            if string::contains(upper, "CODING") then Coding
            else if string::contains(upper, "COMPLEX") then Complex
            else if string::contains(upper, "CREATIVE") then Creative
            else if string::contains(upper, "FACTUAL") then Factual
            else if string::contains(upper, "SIMPLE") then Simple
            else General
        }
    }
}

-- Select the target model based on route classification
--
-- Parameters:
--   target_models: List of (name, model_id) tuples
--   route: The classified route target
--
-- Returns: Selected model identifier
fn select_model(
    target_models: List<(String, String)>,
    route: RouteTarget
) -> String {
    -- Default to first model if no match found
    let default = match list::head(target_models) {
        None => "gpt-4o",
        Some { value: (_, model_id) } => model_id
    };

    -- Find matching model based on route
    let target_name = match route {
        Coding => "coding",
        Complex => "complex",
        Creative => "creative",
        Factual => "factual",
        Simple => "simple",
        General => "general"
    };

    -- Look for matching target in the list
    let find_result = list::find(
        target_models,
        fn(pair: (String, String)) -> Bool {
            let (name, _) = pair;
            string::contains(string::to_lowercase(name), target_name)
        }
    );

    match find_result {
        None => default,
        Some { value: (_, model_id) } => model_id
    }
}

-- Model router workflow
--
-- Uses a classifier model to determine the best target model for a request,
-- then routes the request to that model.
--
-- Parameters:
--   provider: Provider name for both classifier and target models
--   classifier_model: Model to use for routing classification
--   target_models: List of (name, model_id) tuples for routing targets
--   history: Conversation history
--   user_message: The user message to route
--
-- Returns: ChatResponse from the selected target model
--
-- Example:
--   let targets = [
--     ("coding", "claude-sonnet-4"),
--     ("general", "gpt-4o-mini")
--   ];
--   let response = router("openai", "gpt-4o", targets, [], "Write a Rust function to sort a list");
workflow router(
    provider: String,
    classifier_model: String,
    target_models: List<(String, String)>,
    history: List<Message>,
    user_message: String
) -> ChatResponse {
    -- Classify the request
    let route = classify_route(provider, classifier_model, history, user_message);

    -- Select the appropriate model
    let target_model = select_model(target_models, route);

    -- Dispatch to the selected model
    let msg = user(user_message);
    let messages = append(history, msg);
    complete(provider, target_model, messages, None)
}
