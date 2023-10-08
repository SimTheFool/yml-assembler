use super::MixIns;
use crate::utils::result::{AppError, AppResult};
use serde_yaml::{
    value::{Tag, TaggedValue},
    Mapping, Value,
};

impl MixIns {
    const MIX_TAG: &'static str = "!mix";

    pub fn trim(&mut self, val: &Value) -> AppResult<Value> {
        match val {
            Value::Tagged(t) => self.on_tag(t),
            Value::Mapping(map) => self.on_mapping(map),
            Value::Sequence(seq) => self.on_sequence(seq),
            x => Ok(x.clone()),
        }
    }

    fn on_tag(&mut self, val: &TaggedValue) -> AppResult<Value> {
        let tag = &val.tag.to_string();
        let value = &val.value;

        let yml = self.trim(value)?;
        Ok(Value::Tagged(Box::new(TaggedValue {
            tag: Tag::new(tag),
            value: yml,
        })))
    }

    fn on_sequence(&mut self, val: &Vec<Value>) -> AppResult<Value> {
        let mut new_seq: Vec<Value> = vec![];
        for value in val {
            let yml = self.trim(&value)?;
            new_seq.push(yml)
        }
        Ok(Value::Sequence(new_seq))
    }

    fn on_mapping(&mut self, val: &Mapping) -> AppResult<Value> {
        let mut new_map = Mapping::new();
        for (key, value) in val {
            let value = match value {
                Value::Tagged(t) => {
                    let tag = &t.tag.to_string();
                    let value = &t.value;

                    match tag.starts_with(Self::MIX_TAG) {
                        true => {
                            let yml = self.trim(value)?;
                            let key = match key {
                                Value::String(key) => key,
                                _ => {
                                    return Err(AppError::ParseYml(format!(
                                        "Invalid key for mixin: {:?}",
                                        key
                                    )))
                                }
                            };

                            self.entry(key.clone()).or_insert(Vec::new()).push(yml);
                            Ok(None)
                        }
                        false => Ok(Some(Value::Tagged(Box::new(TaggedValue {
                            tag: Tag::new(tag),
                            value: self.trim(value)?,
                        })))),
                    }
                }
                _ => self.trim(value).map(|x| Some(x)),
            }?;

            if let Some(value) = value {
                new_map.insert(key.clone(), value);
            }
        }
        Ok(Value::Mapping(new_map))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_should_apply_trim_yml_mixins() {
        let yml_part: Value = serde_yaml::from_str(
            r#"
            foo: abcde
            hue: !inc::hue
                a: 1
                b: 2
            bar: !mix
                my_mixin
            baz:
                bar: !mix
                    my_mixin_2
            toto: !mix
                tota: !mix
                    my_mixin_3
                totu: what
        "#,
        )
        .unwrap();
        let mut mixin = MixIns::new();

        mixin.trim(&yml_part).unwrap();

        assert_eq!(mixin.len(), 3);

        let barmixin = mixin.get("bar").unwrap();
        let bar_expected_value_1: Value = serde_yaml::from_str(
            r#"
                my_mixin
            "#,
        )
        .unwrap();
        let bar_expected_value_2: Value = serde_yaml::from_str(
            r#"
                my_mixin_2
            "#,
        )
        .unwrap();
        assert_eq!(barmixin, &vec![bar_expected_value_1, bar_expected_value_2]);

        let bazmixin = mixin.get("baz");
        assert_eq!(bazmixin, None);

        let toto = mixin.get("toto").unwrap();
        let toto_expected_value: Value = serde_yaml::from_str(
            r#"
                totu: what
            "#,
        )
        .unwrap();
        assert_eq!(toto, &vec![toto_expected_value]);

        let tota = mixin.get("tota").unwrap();
        let tota_expected_value: Value = serde_yaml::from_str(
            r#"
                my_mixin_3
            "#,
        )
        .unwrap();
        assert_eq!(tota, &vec![tota_expected_value]);
    }

    #[test]
    fn it_not_spread_sequence_into_several_mixins() {
        let yml_part: Value = serde_yaml::from_str(
            r#"
            hue: !mix
                - a: 1
                  b: 2
                - a: 3
                  b: 4
            "#,
        )
        .unwrap();
        let mut mixin = MixIns::new();
        mixin.trim(&yml_part).unwrap();

        let hue_mixin = mixin.get("hue").unwrap();
        let expected_hue_mixin: Vec<Value> = vec![serde_yaml::from_str(
            r#"
                - a: 1
                  b: 2
                - a: 3
                  b: 4
            "#,
        )
        .unwrap()];

        assert_eq!(hue_mixin, &expected_hue_mixin);
    }
}
