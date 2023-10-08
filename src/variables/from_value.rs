use super::Variables;
use crate::utils::result::AppError;
use serde_yaml::Value;

impl TryInto<Variables> for Value {
    type Error = AppError;

    fn try_into(self) -> Result<Variables, Self::Error> {
        let mut variables = Variables::new();

        match self {
            Value::Mapping(map) => {
                for (key, value) in map {
                    let key = match key {
                        Value::String(str) => Ok(str),
                        _ => Err(AppError::ParseYml(format!("Variable key is not a string"))),
                    }?;

                    variables.insert(key, value);
                }
                Ok(variables)
            }
            Value::Null => Ok(variables),
            _ => Err(AppError::ParseYml(format!("Cannot parse as variables"))),
        }
    }
}

#[cfg(test)]
mod test {
    use serde_yaml::Number;

    use super::*;

    #[test]
    fn it_should_convert_mapping_to_variables() {
        let mapping: Value = serde_yaml::from_str(
            r#"
                foo: 3.5
                bar: test
                toto:
                    a: 1
                    b: 2
            "#,
        )
        .unwrap();

        let variables: Variables = mapping.try_into().unwrap();

        let foo_variable = variables.get("foo").unwrap();
        assert_eq!(foo_variable, &Value::Number(Number::from(3.5)));

        let bar_variable = variables.get("bar").unwrap();
        assert_eq!(bar_variable, &Value::String("test".to_string()));

        let toto_variable = variables.get("toto").unwrap();
        let expected_toto_mapping: Value = serde_yaml::from_str(
            r#"
                a: 1
                b: 2
            "#,
        )
        .unwrap();
        assert_eq!(toto_variable, &expected_toto_mapping);
    }

    #[test]
    fn it_should_hold_null_values() {
        let mapping: Value = serde_yaml::from_str(
            r#"
                foo: 3.5
                bar: null
                toto:
                    a: null
                    b: 2
            "#,
        )
        .unwrap();

        let variables: Variables = mapping.try_into().unwrap();

        let foo_variable = variables.get("foo").unwrap();
        assert_eq!(foo_variable, &Value::Number(Number::from(3.5)));

        let bar_variable = variables.get("bar").unwrap();
        assert_eq!(bar_variable, &Value::Null);

        let toto_variable = variables.get("toto").unwrap();
        let expected_toto_mapping: Value = serde_yaml::from_str(
            r#"
                a: null
                b: 2
            "#,
        )
        .unwrap();
        assert_eq!(toto_variable, &expected_toto_mapping);
    }
}
