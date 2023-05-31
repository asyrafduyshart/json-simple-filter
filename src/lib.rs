use serde_json::Number;
use serde_json::Value;

/// A struct representing a filter that can be applied on a JSON Value.
///
/// A filter consists of a field, an operator, and a value to compare with.
/// The field and value can be optionally multiplied by a multiplier.
/// The value to compare with can also be taken from another field.
///
/// # Fields
///
/// * `field` - The name of the field in the JSON Value to apply the filter on.
/// * `operator` - The operator used for comparison.
/// * `value` - The value to compare with.
/// * `value_field` - The name of the field in the JSON Value to take the comparison value from.
/// * `multiplier_field` - The multiplier for the field value.
/// * `multiplier_value` - The multiplier for the comparison value.
///
#[derive(Debug)]
pub struct Filter<'a> {
    field: Option<&'a str>,
    operator: &'a str,
    value: Option<Value>,
    value_field: Option<String>,
    multiplier_field: Option<i64>,
    multiplier_value: Option<i64>,
}

impl<'a> Default for Filter<'a> {
    fn default() -> Self {
        Filter {
            field: None,
            operator: "=",
            value: None,
            value_field: None,
            multiplier_field: None,
            multiplier_value: None,
        }
    }
}

/// Parses a filter string into a list of Filters.
///
/// The function splits the filter string by " AND " to get a list of filter parts.
/// Each part is further split into field, operator, and value.
/// The field and value can optionally have a multiplier and be prefixed with a multiplier followed by "*".
/// The value can also be a reference to a field if it starts with ".".
///
/// # Arguments
///
/// * `filter_string` - The string representation of filters to parse.
///
/// # Returns
///
/// * `Option<Vec<Filter>>` - Returns a list of Filters if the parsing is successful, otherwise returns None.
///
pub fn parse(filter_string: &str) -> Option<Vec<Filter>> {
    let filters = filter_string
        .split(" AND ")
        .map(|filter_part| {
            let parts: Vec<&str> = filter_part.split_whitespace().collect();

            let field_parts: Vec<&str> = parts[0].split('*').collect();
            let multiplier_field = if field_parts.len() == 2 {
                field_parts[0].parse::<i64>().ok()
            } else {
                None
            };
            let field = if field_parts.len() == 1 || multiplier_field.is_some() {
                Some(field_parts[field_parts.len() - 1].trim_start_matches('.'))
            } else {
                None
            };

            let operator = parts[1];

            let value_parts: Vec<&str> = parts[2].split('*').collect();
            let multiplier_value = if value_parts.len() == 2 {
                value_parts[0].parse::<i64>().ok()
            } else {
                None
            };

            let value = value_parts[value_parts.len() - 1].trim_matches('\'');

            let value_field = if value.starts_with('.') {
                Some(value.trim_start_matches('.').to_string())
            } else {
                None
            };

            let value = if value_field.is_none() {
                if let Ok(n) = value.parse::<i64>() {
                    Some(Value::Number(Number::from(n)))
                } else {
                    Some(Value::String(value.to_string()))
                }
            } else {
                None
            };

            Filter {
                field,
                operator,
                value,
                value_field,
                multiplier_field,
                multiplier_value,
            }
        })
        .collect();
    Some(filters)
}

/// Applies a set of filters on a JSON Value and returns whether the Value passes the filters.
///
/// The function iterates over a list of filters and applies each filter on the Value `v`.
/// The field to be compared is extracted from the Value, based on the `field` attribute of the filter.
/// The value to compare with is determined based on the `value_field` or `value` attributes of the filter.
///
/// The comparison is done either as a string comparison or as a number comparison,
/// depending on the types of the extracted field and value.
/// For number comparisons, a multiplier can be applied to the field or value.
///
/// If a filter comparison is unsuccessful, the function immediately returns `false`.
/// If all filter comparisons are successful, the function returns `true`.
///
/// # Arguments
///
/// * `v` - The JSON Value to apply the filters on.
/// * `filters` - A slice of Filters to apply on the Value.
///
/// # Returns
///
/// * `bool` - Returns `true` if the Value `v` passes all the filters, otherwise returns `false`.
///
pub fn apply(v: &Value, filters: &[Filter]) -> bool {
    for filter in filters {
        // The field we're comparing is taken from the JSON value.
        let f = filter.field.as_deref().and_then(|field| v.get(field));
        let f_is_number = matches!(f, Some(Value::Number(_)));

        // If the filter has a value_field, we take the value to compare from the JSON value.
        // If there is no value_field, we use the value directly.
        let value = filter.value_field.as_deref().and_then(|vf| v.get(vf));

        // Then we perform the comparison according to the operator in the filter.
        // If both are strings, compare them as strings. If not, try to compare as numbers.
        let comparison = if !f_is_number {
            let f_str = f.and_then(|val| val.as_str());
            // if value id true get from value, if not get from value_filed
            let value_str = if filter.value.is_some() {
                filter.value.as_ref().and_then(|val| val.as_str())
            } else {
                filter
                    .value_field
                    .as_deref()
                    .and_then(|vf| v.get(vf))
                    .and_then(|val| val.as_str())
            };
            match (f_str, value_str) {
                (Some(f_str), Some(value_str)) => match filter.operator {
                    "=" => f_str == value_str,
                    "!=" => f_str != value_str,
                    _ => false, // Unknown operator for string comparisons
                },
                _ => false, // In case there's a mismatch in type (one is number and the other is string)
            }
        } else {
            // Now we multiply it by its multiplier if there is one.
            let f = if let (Some(mult), Some(val)) = (filter.multiplier_field, f) {
                val.as_i64().map(|v| v * mult)
            } else {
                f.and_then(|val| val.as_i64())
            };

            let value = if let (Some(mult), Some(val)) = (filter.multiplier_value, value) {
                val.as_i64().map(|v| v * mult)
            } else {
                value
                    .and_then(|val| val.as_i64())
                    .or_else(|| filter.value.clone().and_then(|val| val.as_i64()))
            };

            match (f, value) {
                (Some(f), Some(value)) => match filter.operator {
                    "=" => f == value,
                    "!=" => f != value,
                    ">=" => f >= value,
                    ">" => f > value,
                    "<=" => f <= value,
                    "<" => f < value,
                    _ => false, // Unknown operator
                },
                _ => false, // In case there's a mismatch in type (one is number and the other is string)
            }
        };

        // If the comparison is false, we return false immediately.
        if !comparison {
            return false;
        }
    }
    // If none of the filters returned false, we return true.
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse() {
        let filter_string = ".field = 'hello' AND .value >= 20";
        let filters = parse(filter_string).unwrap();
        assert_eq!(filters.len(), 2);
        assert_eq!(filters[0].field, Some("field"));
        assert_eq!(filters[0].operator, "=");
        assert_eq!(filters[0].value, Some(json!("hello")));
        assert_eq!(filters[1].field, Some("value"));
        assert_eq!(filters[1].operator, ">=");
        assert_eq!(filters[1].value, Some(json!(20)));
    }

    #[test]
    fn test_apply() {
        let v = json!({ "field": 100, "hello": "world" });
        let filters = vec![
            Filter {
                field: Some("field"),
                operator: ">",
                value: Some(json!(50)),
                ..Default::default()
            },
            Filter {
                field: Some("hello"),
                operator: "=",
                value: Some(json!("world")),
                ..Default::default()
            },
        ];
        assert!(apply(&v, &filters));
    }
}
