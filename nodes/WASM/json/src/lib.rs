use serde_json::Value;

#[no_mangle]
extern fn main(json: &String, fields: &String) -> Option<Box<String>> {
    let value: Value = serde_json::from_str(json).ok()?;
    let fields = fields.split("->").collect::<Vec<_>>();
    match fields.len() {
        0 => None,
        1 => value.get(fields[0]).map(|it| Box::new(it.to_string())),
        _ => {
            let mut output = value.get(fields[0]);
            for field in &fields[1..] {
                if let Some(ref mut value) = output {
                    match field.parse::<usize>() {
                        Ok(index) => *value = &value[index],
                        _ => *value = &value[field],
                    }
                }
            }
            Some(Box::new(output?.to_string()))
        }
    }
}
