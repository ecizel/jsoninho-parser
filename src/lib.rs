// a implementar
// parser:
// fazer ser utilizavel
// montar o macro de deserealização (acessar direto, sem pattern match)

use indexmap::IndexMap;
use std::iter::Peekable;

#[derive(Debug, PartialEq)]
pub enum JsonValue {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(IndexMap<String, JsonValue>),
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
    // values
    True,
    False,
    Null,
    String(String),
    Number(String), // facilita trabalho do lexer e do parser
}

fn remove_whitespaces(content_iter: &mut impl Iterator<Item = char>) -> Option<char> {
    loop {
        let c = content_iter.next()?;
        match c {
            ' ' | '\n' | '\t' | '\r' => continue,
            _ => return Some(c),
        }
    }
}

pub fn tokenize_json(content: &str) -> Result<Vec<JsonToken>, &'static str> {
    let mut tokens: Vec<JsonToken> = vec![];

    let mut content_iter = content.chars().peekable();

    while let Some(c) = remove_whitespaces(&mut content_iter) {
        match c {
            // structure
            '{' => tokens.push(JsonToken::OpenCurlyBkt),
            '}' => tokens.push(JsonToken::CloseCurlyBkt),
            '[' => tokens.push(JsonToken::OpenSquareBkt),
            ']' => tokens.push(JsonToken::CloseSquareBkt),
            ',' => tokens.push(JsonToken::Comma),
            ':' => tokens.push(JsonToken::Colon),
            '"' => tokens.push(new_token_string(&mut content_iter)?),
            '0'..='9' | '-' => tokens.push(new_token_number(c, &mut content_iter)?),
            't' | 'f' | 'n' => tokens.push(new_token_boolean_or_null(c, &mut content_iter)?),
            _ => return Err("unexpected character in tokenizer"),
        };
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
        Some(t) => match t {
            JsonToken::OpenCurlyBkt => new_object(&mut tokens_iter)?,
            JsonToken::OpenSquareBkt => new_array(&mut tokens_iter)?,
            _ => return Err("invalid start of json"),
        },
        None => return Err("empty json"),
    };

    if tokens_iter.next().is_some() {
        return Err("trailing data after json structure");
    }

    Ok(json)
}

fn new_array(tokens_iter: &mut impl Iterator<Item = JsonToken>) -> Result<JsonValue, &'static str> {
    let mut after_first = false;
    let mut after_comma = false;
    let mut vec_aux = vec![];

    loop {
        match tokens_iter.next() {
            Some(t) => match t {
                JsonToken::CloseSquareBkt => {
                    if after_comma {
                        return Err("comma before end of array")
                    }
                    break
                },
                JsonToken::Comma => {
                    if after_first && !after_comma {
                        after_comma = true;
                        continue;
                    } else {
                        return Err("invalid comma in array");
                    }
                }
                _ => {
                    if after_first && !after_comma {
                        return Err("should be comma between array items")
                    }
                    after_first = true;
                    after_comma = false;
                    vec_aux.push(new_value(t, tokens_iter)?);
                }
            },
            None => return Err("json ended abruptly in array"),
        }
    }
    Ok(JsonValue::Array(vec_aux))
}

fn new_object(
    tokens_iter: &mut impl Iterator<Item = JsonToken>,
) -> Result<JsonValue, &'static str> {
    let mut after_first = false;
    let mut after_comma = false;
    let mut hash_aux = IndexMap::new();

    loop {
        match tokens_iter.next() {
            Some(c) => match c {
                JsonToken::CloseCurlyBkt => {
                    if after_comma {
                        return Err("comma before end of object")
                    }
                    break
                },
                JsonToken::Comma => {
                    if after_first && !after_comma {
                        after_comma = true;
                        continue;
                    } else {
                        return Err("invalid comma in object");
                    }
                }
                JsonToken::String(s) => {
                    if after_first && !after_comma {
                        return Err("should be comma between objetct values")
                    }
                    after_first = true;
                    after_comma = false;

                    if tokens_iter.next().is_some_and(|c| c != JsonToken::Colon) {
                        return Err("unexpected character where should be colon in object");
                    }

                    let value = match tokens_iter.next() {
                        Some(c) => new_value(c, tokens_iter)?,
                        None => return Err("unexpected end of object"),
                    };

                    hash_aux.insert(s, value);
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
        JsonToken::OpenCurlyBkt => new_object(token_iter),
        JsonToken::OpenSquareBkt => new_array(token_iter),
        JsonToken::String(s) => Ok(JsonValue::String(s)),
        JsonToken::Number(n) => {
            let parsed = n.parse::<f64>().map_err(|_| "invalid number format")?;
            Ok(JsonValue::Number(parsed))
        }
        JsonToken::Null => Ok(JsonValue::Null),
        JsonToken::True => Ok(JsonValue::Boolean(true)),
        JsonToken::False => Ok(JsonValue::Boolean(false)),
        _ => Err("unexpected character in value"),
    }
}

fn new_token_string<I>(
    content_iter: &mut Peekable<I>,
) -> Result<JsonToken, &'static str> 
where I : Iterator<Item = char>
{
    let mut texted = String::new();
    let mut closed = false;

    while let Some(c) = content_iter.next() {
        if c == '"' {
            closed = true;
            break;
        }
        
        if c == '\\' {
            match content_iter.next() {
                Some('n') => texted.push('\n'),
                Some('t') => texted.push('\t'),
                Some('r') => texted.push('\r'),
                Some('b') => texted.push('\u{0008}'), // backspace
                Some('f') => texted.push('\u{000C}'), // formfeed
                Some('"') => texted.push('"'),
                Some('\\') => texted.push('\\'),
                Some('/') => texted.push('/'),
                // 4 hex digits
                Some('u') => {
                    let mut hex_val = 0u32;
                    for _ in 0..4 {
                        match content_iter.next() {
                            Some(ch) if ch.is_ascii_hexdigit() => {
                                hex_val = hex_val * 16 + ch.to_digit(16).unwrap();
                            }
                            _ => return Err("invalid unicode escape sequence"),
                        }
                    }
                    let ch = char::from_u32(hex_val).ok_or("invalid unicode scalar value")?;
                    texted.push(ch);
                }
                Some(_) => return Err("invalid escape sequence in string"),
                None => return Err("string ended abruptly after escape"),
            }
        } else {
            texted.push(c);
        }
    }

    if !closed {
        return Err("string ended abruptly without closing quote");
    }

    Ok(JsonToken::String(texted))
}

fn new_token_number<I>(
    start: char,
    content_iter: &mut Peekable<I>,
) -> Result<JsonToken, &'static str>
where
    I: Iterator<Item = char>,
{
    let mut numbered = String::from(start);

    // a logica tem como base que tudo vai ser uma string
    // e quem vai transformar em número é a linguagem
    // e os 2 juntos formam a logica do JSON...
    let mut started = start != '-';
    let mut zero = start == '0';
    let mut has_dot = false;
    let mut in_e = false;
    let mut e_started = false;
    
    let mut endable = started;

    while let Some(&p) = content_iter.peek() {
        // numero pode sempre
        if p.is_ascii_digit() {
            // se in_e nao pode mais (-) ou (+)
            if in_e {
                e_started = true;
            }
            // logica para nao ter 0s repetidos no inicio
            // ou 0 e depois um numero
            if zero {
                if p == '0' {
                    return Err("repeated zeroes in start of number")
                } else {
                    return Err("no number should be after starting zero")
                }
            }
            if !started {
                zero = p == '0';
            } else {
                zero = false;
            }

            endable = true;
            // se o 'start' tiver sido (-)
            started = true;
            numbered.push(p);
            content_iter.next();
        // (.) precisa ter começado e nao pode ser no e
        } else if p == '.' {
            if has_dot {
                return Err("repeated dot in number");
            }
            if !started {
                return Err("should be a number before dot");
            }
            if in_e {
                return Err("invalid dot in e");
            }
            
            endable = false;
            zero = false;
            has_dot = true;
            numbered.push(p);
            content_iter.next();
        } else if p == 'e' || p == 'E' {
            if in_e {
                return Err("repeated e in number");
            }

            endable = false;
            zero = false;
            in_e = true;
            numbered.push(p);
            content_iter.next();
        } else if p == '+' || p == '-' {
            if !in_e {
                return Err("invalid plus/minus sign");
            }
            if e_started {
                return Err("repeated plus/minus sign in e");
            }

            endable = false;
            e_started = true;
            numbered.push(p);
            content_iter.next();
        } else {
            break;
        }
    }

    if !endable {
        return Err("unexpected number ending")
    }
    Ok(JsonToken::Number(numbered))
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
            Ok(JsonToken::True)
        }

        'f' => {
            let is_valid = content_iter.next() == Some('a')
                && content_iter.next() == Some('l')
                && content_iter.next() == Some('s')
                && content_iter.next() == Some('e');

            if !is_valid {
                return Err("invalid json format in 'false'");
            }
            Ok(JsonToken::False)
        }

        'n' => {
            let is_valid = content_iter.next() == Some('u')
                && content_iter.next() == Some('l')
                && content_iter.next() == Some('l');

            if !is_valid {
                return Err("invalid json format in 'null'");
            }
            Ok(JsonToken::Null)
        }
        _ => Err("this should not appear! ERROR IN BOOLEAN-NULL FUNCTION"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==========================================
    // 1. TESTES DE SUCESSO (Caminho Feliz)
    // ==========================================

    #[test]
    fn test_success_primitives_and_types() {
        let json_str = r#"{
            "name": "Rust\nLang", 
            "version": 2026.5, 
            "active": true, 
            "nothing": null, 
            "score": -1e-2,
            "zero": 0
        }"#;

        let parsed = parse_json(json_str).unwrap();

        if let JsonValue::Object(map) = parsed {
            assert_eq!(map.get("active"), Some(&JsonValue::Boolean(true)));
            assert_eq!(map.get("nothing"), Some(&JsonValue::Null));
            
            if let Some(JsonValue::Number(ver)) = map.get("version") {
                assert_eq!(*ver, 2026.5);
            } else {
                panic!("Expected version number");
            }

            if let Some(JsonValue::String(name)) = map.get("name") {
                assert_eq!(name, "Rust\nLang");
            } else {
                panic!("Expected name string");
            }
        } else {
            panic!("Expected root to be an object");
        }
    }

    #[test]
    fn test_success_nested_structures() {
        let json_str = r#"{
            "users": [
                {"id": 1, "tags": ["rust", "json"]},
                {"id": 2, "tags": []}
            ],
            "meta": {"total": 2}
        }"#;

        assert!(parse_json(json_str).is_ok());
    }

    #[test]
    fn test_success_empty_structures() {
        assert!(parse_json("{}").is_ok());
        assert!(parse_json("[]").is_ok());
        assert!(parse_json(r#"{"empty_arr": [], "empty_obj": {}}"#).is_ok());
    }

    // ==========================================
    // 2. TESTES DE ERRO (Estrutura e Sintaxe)
    // ==========================================

    #[test]
    fn test_error_empty_and_trailing() {
        // JSON vazio ou apenas com espaços
        assert_eq!(parse_json("   "), Err("empty json"));

        // Dados extras após a estrutura JSON principal
        assert_eq!(parse_json(r#"{"a": 1} 123"#), Err("trailing data after json structure"));
        assert_eq!(parse_json(r#"{"a": 1} extra data"#), Err("unexpected character in tokenizer"));
    }

    #[test]
    fn test_error_invalid_root() {
        // O JSON deve começar obrigatoriamente com Objeto ou Array no root
        assert_eq!(parse_json("123"), Err("invalid start of json"));
        assert_eq!(parse_json(r#""just a string""#), Err("invalid start of json"));
        assert_eq!(parse_json("true"), Err("invalid start of json"));
    }

    #[test]
    fn test_error_missing_syntax() {
        // Falta de dois-pontos em objetos
        assert!(parse_json(r#"{"a" 1}"#).is_err());

        // Falta de vírgula entre elementos de arrays e objetos (grudados)
        assert!(parse_json("[1 2]").is_err());
        assert!(parse_json(r#"{"a": 1 "b": 2}"#).is_err());
    }

    #[test]
    fn test_error_trailing_and_double_commas() {
        // Vírgulas sobrando no final (Trailing comma)
        assert!(parse_json("[1, 2,]").is_err());
        assert!(parse_json(r#"{"a": 1,}"#).is_err());

        // Vírgulas duplas (Double comma)
        assert!(parse_json("[1,, 2]").is_err());
        assert!(parse_json(r#"{"a": 1,, "b": 2}"#).is_err());
    }

    #[test]
    fn test_error_malformed_numbers() {
        // Zeros à esquerda inválidos
        assert!(parse_json("[01]").is_err());
        assert!(parse_json("[00]").is_err());

        // Múltiplos pontos decimais
        assert!(parse_json("[1.2.3]").is_err());

        // Terminações de número inválidas (ex: ponto ou expoente sobrando sem dígitos)
        assert!(parse_json("[1.]").is_err());
        assert!(parse_json("[1e]").is_err());
        assert!(parse_json("[1e+]").is_err());
    }

    #[test]
    fn test_error_unclosed_string() {
        // String sem fechar as aspas
        assert!(parse_json(r#"{"a": "unclosed}"#).is_err());
    }

    #[test]
    fn test_error_abrupt_end() {
        // Fim de arquivo abrupto no meio de uma estrutura
        assert!(parse_json(r#"{"a": "#).is_err());
        assert!(parse_json("[1, 2").is_err());
        assert!(parse_json(r#"{"a": 1"#).is_err());
    }
    
    // ==========================================
    // 3. TESTES DE SUCESSO (Escapes Avançados)
    // ==========================================

    #[test]
    fn test_success_advanced_string_escapes() {
        // Testa \b (backspace), \f (formfeed), \/ (solidus) e \uXXXX (Unicode, ex: ©)
        let json_str = r#"{
            "escaped": "Line1\bLine2\fLine3\/path\u00A9"
        }"#;

        let parsed = parse_json(json_str).unwrap();

        if let JsonValue::Object(map) = parsed {
            if let Some(JsonValue::String(s)) = map.get("escaped") {
                // \u{0008} é o backspace, \u{000C} o formfeed, e \u{00A9} o '©'
                let expected = "Line1\u{0008}Line2\u{000C}Line3/path©";
                assert_eq!(s, expected);
            } else {
                panic!("Expected string value for 'escaped'");
            }
        } else {
            panic!("Expected root to be an object");
        }
    }

    // ==========================================
    // 4. TESTES DE ERRO (Escapes e Unicode Inválidos)
    // ==========================================

    #[test]
    fn test_error_invalid_string_escapes() {
        // Carater de escape desconhecido (\x não existe no JSON)
        assert!(parse_json(r#"{"a": "invalid \x escape"}"#).is_err());

        // Sequência Unicode incompleta (menos de 4 dígitos)
        assert!(parse_json(r#"{"a": "short unicode \u123"}"#).is_err());

        // Sequência Unicode com carateres não hexadecimais (o 'Z')
        assert!(parse_json(r#"{"a": "bad hex \u123Z"}"#).is_err());

        // String terminada abruptamente logo após a barra invertida de escape
        assert!(parse_json(r#"{"a": "ends with slash \"#).is_err());
    }
}