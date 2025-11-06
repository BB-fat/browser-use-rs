use crate::error::{BrowserError, Result};
use crate::tools::{Tool, ToolContext, ToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvaluateParams {
    /// JavaScript code to execute
    pub code: String,
    
    /// Wait for promise resolution (default: false)
    #[serde(default)]
    pub await_promise: bool,
}

pub struct EvaluateTool;

impl Tool for EvaluateTool {
    fn name(&self) -> &str {
        "evaluate"
    }

    fn description(&self) -> &str {
        "Execute JavaScript code in the browser context"
    }

    fn parameters_schema(&self) -> Value {
        serde_json::to_value(schemars::schema_for!(EvaluateParams)).unwrap_or_default()
    }

    fn execute(&self, params: Value, context: &mut ToolContext) -> Result<ToolResult> {
        let params: EvaluateParams = serde_json::from_value(params)
            .map_err(|e| BrowserError::InvalidArgument(e.to_string()))?;

        let result = context.session.tab()
            .evaluate(&params.code, params.await_promise)
            .map_err(|e| BrowserError::EvaluationFailed(e.to_string()))?;

        let result_value = result.value.unwrap_or(Value::Null);

        Ok(ToolResult::success_with(serde_json::json!({
            "result": result_value
        })))
    }
}
