// implementar resto do linter (validar):
// - objetovs, os dois pontos (e com qualquer coisa dentro)
// - listas/objetos em listas/objetos
// numeros (inteiros e com ponto e vergiula)

// parser:
// juntar com o JsonValue
// retornar o objeto completo (estruturado)
// fazer ser utilizavel

// checar erros de fim de json invalido

#[derive(Debug)]
enum JsonValue {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(std::collections::HashMap<String, JsonValue>),
}

fn remove_whitespaces(content_iter: &mut impl Iterator<Item = char>) -> Option<char> {
    loop {
        match content_iter.next() {
            Some(c) => match c {
                ' ' | '\n' | '\t' | '\r' => continue,
                _ => return Some(c),
            },
            None => return None,
        }
    }
}

pub fn parse_json(content: String) -> Result<(), &'static str> {
    let mut content_iter = content.chars();

    let c = remove_whitespaces(&mut content_iter);

    match c {
        Some(c) => match c {
            '[' => new_array(&mut content_iter)?,
            '{' => new_object(&mut content_iter)?,
            _ => return Err("invalid start of json"),
        },
        None => return Err("empty json"),
    };

    Ok(())
}

fn new_array(content_iter: &mut impl Iterator<Item = char>) -> Result<(), &'static str> {
    println!("start_of_array");
    let mut after_first = false;
    loop {
        match remove_whitespaces(content_iter) {
            Some(c) => match c {
                ']' => {
                    println!("end_of_array");
                    break;
                }
                ',' => {
                    if after_first {
                        continue;
                    } else {
                        return Err("invalid comma in array");
                    }
                }
                _ => {
                    after_first = true;
                    new_value(c, content_iter)?;
                }
            },
            None => return Err("json ended abruptly in array"),
        }
    }
    Ok(())
}

fn new_object(content_iter: &mut impl Iterator<Item = char>) -> Result<(), &'static str> {
    println!("start_of_object");
    let mut after_first = false;
    loop {
        match remove_whitespaces(content_iter) {
            Some(c) => match c {
                '}' => {
                    println!("end_of_object");
                    break;
                }
                '"' => {
                    after_first = true;
                    print!("key -> ");
                    new_string(content_iter)?;

                    let colon =
                        remove_whitespaces(content_iter).ok_or("unexpected end of object 1")?;

                    if colon != ':' {
                        return Err("unexpected character in object 2");
                    }

                    print!("value -> ");
                    let value_char =
                        remove_whitespaces(content_iter).ok_or("unexpected end of object 2")?;

                    new_value(value_char, content_iter)?;
                }
                ',' => {
                    if after_first {
                        continue;
                    } else {
                        return Err("invalid comma in object");
                    }
                }
                _ => return Err("unexpected character in object 1"),
            },
            None => return Err("json ended abruptly in object"),
        }
    }
    Ok(())
}

fn new_value(
    start: char,
    content_iter: &mut impl Iterator<Item = char>,
) -> Result<(), &'static str> {
    match start {
        '\"' => new_string(content_iter)?,
        '{' => new_object(content_iter)?,
        '[' => new_array(content_iter)?,
        't' | 'f' | 'n' => new_boolean_or_null(start, content_iter)?,
        _ => return Err("unexpected character in value"),
    }

    Ok(())
}

// A IMPLEMENTAR: caracteres especiais
fn new_string(content_iter: &mut impl Iterator<Item = char>) -> Result<(), &'static str> {
    let texted: String = content_iter
        .by_ref()
        .take_while(|&c| c != '\"')
        .collect();
    println!("string: {}", texted);
    Ok(())
}

fn new_boolean_or_null(
    start: char,
    content_iter: &mut impl Iterator<Item = char>,
) -> Result<(), &'static str> {
    match start {
        't' => {
            let is_valid = content_iter.next() == Some('r')
                && content_iter.next() == Some('u')
                && content_iter.next() == Some('e');

            if !is_valid {
                return Err("invalid json format in 'true'");
            }
            println!("true");
        }

        'f' => {
            let is_valid = content_iter.next() == Some('a')
                && content_iter.next() == Some('l')
                && content_iter.next() == Some('s')
                && content_iter.next() == Some('e');

            if !is_valid {
                return Err("invalid json format in 'false'");
            }
            println!("false");
        }

        'n' => {
            let is_valid = content_iter.next() == Some('u')
                && content_iter.next() == Some('l')
                && content_iter.next() == Some('l');

            if !is_valid {
                return Err("invalid json format in 'null'");
            }
            println!("null");
        }
        _ => return Err("this should not appear! ERROR IN BOOLEAN-NULL FUNCTION"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn read_filepath() {}
}
