// implementar resto do linter (validar):
// numeros (inteiros e com ponto e vergiula)
// caracteres especiais, inicio \...

// parser:
// fazer ser utilizavel
// montar o macro de deserealização (acessar direto, sem pattern match)

#[derive(Debug)]
pub enum JsonValue {
    Null,
    Boolean(bool),
    // Number(f64), usar nos match '0'..='9' parece que funciona...
    String(String),
    Array(Vec<JsonValue>),
    Object(indexmap::IndexMap<String, JsonValue>),
}

#[derive(Debug, PartialEq)]
pub enum JsonToken {
    // structure
    OpenCurlyBkt,
    CloseCurlyBkt,
    OpenSquareBkt,
    CloseSquareBkt,
    Comma,
    Colon,
    // values (no numbers or special characters)
    True,
    False,
    Null,
    String(String),
    // Number(String) // facilita trabalho do lexer e do parser
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

pub fn tokenize_json(content: &str) -> Result<Vec<JsonToken>, &'static str> {
    let mut tokens: Vec<JsonToken> = vec![];

    let mut content_iter = content.chars();

    loop {
        match remove_whitespaces(&mut content_iter) {
            Some(c) => match c {
                // structure
                '{' => tokens.push(JsonToken::OpenCurlyBkt),
                '}' => tokens.push(JsonToken::CloseCurlyBkt),
                '[' => tokens.push(JsonToken::OpenSquareBkt),
                ']' => tokens.push(JsonToken::CloseSquareBkt),
                ',' => tokens.push(JsonToken::Comma),
                ':' => tokens.push(JsonToken::Colon),
                '"' => tokens.push(new_token_string(&mut content_iter)?),
                't' | 'f' | 'n' => tokens.push(new_token_boolean_or_null(c, &mut content_iter)?),
                _ => return Err("unexpected character in tokenizer")

            }
            None => break
        }
    }

    Ok(tokens) 
}

pub fn parse_json(content: &str) -> Result<JsonValue, &'static str> {
    let tokens = tokenize_json(content)?;

    // logica toda está aqui
    // no futuro separar fuunção especifica
    // e adicionar o macro para enquadrar na estrutura do usuario
    let mut tokens_iter = tokens.into_iter(); 
    let json = match tokens_iter.next() {
        Some(t) => {
            match t {
                JsonToken::OpenCurlyBkt => new_object(&mut tokens_iter)?,
                JsonToken::OpenSquareBkt => new_array(&mut tokens_iter)?,
                _ => return Err("invalid start of json")
            }
        }
        None => return Err("empty json")
    };

    if tokens_iter.next().is_some() {
        return Err("trailing data after json structure");
    }

    Ok(json)
}

fn new_array(tokens_iter: &mut impl Iterator<Item = JsonToken>) -> Result<JsonValue, &'static str> {
    let mut after_first = false;
    let mut vec_aux = vec![];
    
    loop {
        match tokens_iter.next() {
            Some(t) => match t {
                JsonToken::CloseSquareBkt => break,
                JsonToken::Comma => {
                    if after_first {
                        continue;
                    } else {
                        return Err("invalid comma in array");
                    }
                }
                _ => {
                    after_first = true;
                    vec_aux.push(new_value(t, tokens_iter)?);
                }
            },
            None => return Err("json ended abruptly in array"),
        }
    }
    Ok(JsonValue::Array(vec_aux))
}

fn new_object(tokens_iter: &mut impl Iterator<Item = JsonToken>) -> Result<JsonValue, &'static str> {
    let mut after_first = false;
    let mut hash_aux = indexmap::IndexMap::new();
    
    loop {
        match tokens_iter.next() {
            Some(c) => match c {
                JsonToken::CloseCurlyBkt => break,
                JsonToken::String(s) => {
                    after_first = true;
                    
                    if tokens_iter.next().is_some_and(|c| c != JsonToken::Colon) {
                        return Err("unexpected character where should be colon in object")
                    }

                    let value = match tokens_iter.next() {
                        Some(c) => new_value(c, tokens_iter)?,
                        None => return Err("unexpected end of object")
                    };
                    
                    hash_aux.insert(s, value);
                }
                JsonToken::Comma => {
                    if after_first {
                        continue;
                    } else {
                        return Err("invalid comma in object");
                    }
                }
                _ => return Err("unexpected character in object"),
            },
            None => return Err("json ended abruptly in object"),
        }
    }

    Ok(JsonValue::Object(hash_aux))
}

fn new_value(
    token: JsonToken,
    token_iter: &mut impl Iterator<Item = JsonToken>,
) -> Result<JsonValue, &'static str> {
    match token {
        JsonToken::OpenCurlyBkt => Ok(new_object(token_iter))?,
        JsonToken::OpenSquareBkt => Ok(new_array(token_iter))?,
        // values
        JsonToken::String(s) => Ok(JsonValue::String(s)),
        JsonToken::Null => Ok(JsonValue::Null),
        JsonToken::True => Ok(JsonValue::Boolean(true)),
        JsonToken::False => Ok(JsonValue::Boolean(false)),
        _ => return Err("unexpected character in value"),
    }
}

// A IMPLEMENTAR: caracteres especiais
fn new_token_string(content_iter: &mut impl Iterator<Item = char>) -> Result<JsonToken, &'static str> {
    let texted: String = content_iter
        .by_ref()
        .take_while(|&c| c != '\"')
        .collect();

    Ok(JsonToken::String(texted))
}

fn new_token_boolean_or_null(
    start: char,
    content_iter: &mut impl Iterator<Item = char>,
) -> Result<JsonToken, &'static str> {
    match start {
        't' => {
            let is_valid = content_iter.next() == Some('r')
                && content_iter.next() == Some('u')
                && content_iter.next() == Some('e');

            if !is_valid {
                return Err("invalid json format in 'true'");
            }
            return Ok(JsonToken::True);
        }

        'f' => {
            let is_valid = content_iter.next() == Some('a')
                && content_iter.next() == Some('l')
                && content_iter.next() == Some('s')
                && content_iter.next() == Some('e');

            if !is_valid {
                return Err("invalid json format in 'false'");
            }
            return Ok(JsonToken::False);
        }

        'n' => {
            let is_valid = content_iter.next() == Some('u')
                && content_iter.next() == Some('l')
                && content_iter.next() == Some('l');

            if !is_valid {
                return Err("invalid json format in 'null'");
            }
            return Ok(JsonToken::Null);
        }
        _ => return Err("this should not appear! ERROR IN BOOLEAN-NULL FUNCTION"),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn read_filepath() {}
}
