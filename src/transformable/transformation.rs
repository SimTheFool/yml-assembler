use evalexpr::{
    Context, ContextWithMutableVariables, EmptyContextWithBuiltinFunctions, EvalexprResult, Value,
};

use crate::utils::result::{AppError, AppResult};

use super::TransformableList;

impl Context for TransformableList {
    fn are_builtin_functions_disabled(&self) -> bool {
        false
    }

    fn set_builtin_functions_disabled(&mut self, _: bool) -> EvalexprResult<()> {
        Ok(())
    }

    fn call_function(
        &self,
        _identifier: &str,
        _arg: &evalexpr::Value,
    ) -> EvalexprResult<evalexpr::Value> {
        let ctx = EmptyContextWithBuiltinFunctions {};
        ctx.call_function(_identifier, _arg)
    }

    fn get_value(&self, identifier: &str) -> Option<&Value> {
        self.get(identifier)
    }
}

impl ContextWithMutableVariables for TransformableList {
    fn set_value(&mut self, _identifier: String, _value: Value) -> EvalexprResult<()> {
        println!("Setting {} to {:?}", _identifier, _value);
        self.set(_identifier, _value);
        Ok(())
    }
}

impl TransformableList {
    pub fn transform(&mut self) -> AppResult<&Self> {
        let operations = match self.get_operations() {
            Some(operations) => operations,
            None => return Ok(self),
        };

        for oper in operations {
            evalexpr::eval_with_context_mut(&oper, self)
                .map_err(|e| AppError::ApplyFormula(e.to_string()))?;
        }

        Ok(self)
    }
}

#[test]
fn it_should_apply_transformations() {
    let operations = vec![
        "a.0.u = a.0.u + 1".to_string(),
        "a.0.v = a.0.v || true".to_string(),
    ];

    let mut transf_list = TransformableList::new(Some(operations));
    transf_list.set("a.0.u".to_string(), Value::Float(1.0));
    transf_list.set("a.0.v".to_string(), Value::Boolean(false));
    transf_list.set("b.x".to_string(), Value::Float(3.0));

    transf_list.transform().unwrap();

    assert_eq!(transf_list.get("a.0.u").unwrap(), &Value::Float(2.0));
    assert_eq!(transf_list.get("a.0.v").unwrap(), &Value::Boolean(true));
    assert_eq!(transf_list.get("b.x").unwrap(), &Value::Float(3.0));
}

#[test]
fn it_should_ceil() {
    let operations = vec!["var = ceil(var / 2.0)".to_string()];

    let mut transf_list = TransformableList::new(Some(operations));
    transf_list.set("var".to_string(), Value::Int(3));

    transf_list.transform().unwrap();

    assert_eq!(transf_list.get("var").unwrap(), &Value::Float(2.0));
}
