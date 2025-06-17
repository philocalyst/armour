use std::error::Error;
use std::fmt;

#[derive(Clone, Debug)]
pub struct LuaFunctionSpec {
    name: String,
    description: Option<String>,
    params: Vec<LuaParam>,
}

#[derive(Clone, Debug)]
pub struct LuaParam {
    name: String,
    ltype: Option<String>,
    description: Option<String>,
}

#[derive(Debug)]
pub struct ParseError {
    msg: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "parse failed: {}", self.msg)
    }
}

impl Error for ParseError {}

const MAIN_NAME: &str = "build_badge";

pub fn parse_lua_docs(lua_script: &str) -> Result<LuaFunctionSpec, ParseError> {
    let mut cur_region = Vec::new();

    for line in lua_script.lines() {
        let line = line.trim();
        if line.len() == 0 {
            continue;
        } else if line.starts_with("--") {
            cur_region.push(line.strip_prefix("--").unwrap().trim());
        } else if cur_region.len() > 0 {
            match parse_comment_region(&cur_region)? {
                Some(lfs) => {
                    if lfs.name == MAIN_NAME {
                        return Ok(lfs);
                    }
                }
                None => {}
            }
            cur_region.clear();
        }
    }

    Err(ParseError {
        msg: format!("Expected docstring '@function {MAIN_NAME}'"),
    })
}

fn parse_comment_region(lines: &Vec<&str>) -> Result<Option<LuaFunctionSpec>, ParseError> {
    let mut name: Option<String> = None;
    let mut description: Option<String> = None;
    let mut params = Vec::new();

    for line in lines {
        let chunks: Vec<&str> = line.split(char::is_whitespace).collect();
        match chunks[0] {
            "@function" => {
                if chunks.len() < 2 {
                    return Err(ParseError {
                        msg: format!("Expected function name after '@function' in {line}"),
                    });
                }
                name = Some(chunks[1].to_string());
                description = str_from_chunks(&chunks[2..]);
            }
            "@param" => {
                if chunks.len() < 2 {
                    return Err(ParseError {
                        msg: format!("Expected param name after '@param' in {line}"),
                    });
                }
                params.push(LuaParam {
                    name: chunks[1].to_string(),
                    ltype: None,
                    description: str_from_chunks(&chunks[2..]),
                })
            }
            "@tparam" => {
                if chunks.len() < 3 {
                    return Err(ParseError {
                        msg: format!("Expected param type and name after '@tparam' in {line}"),
                    });
                }
                let ltype = chunks[2].to_string();
                params.push(LuaParam {
                    name: chunks[1].to_string(),
                    ltype: Some(ltype),
                    description: str_from_chunks(&chunks[2..]),
                })
            }
            _ => continue,
        }
    }

    match name {
        Some(name) => Ok(Some(LuaFunctionSpec {
            name,
            description,
            params,
        })),
        None => Ok(None),
    }
}

fn str_from_chunks(chunks: &[&str]) -> Option<String> {
    let desc = chunks.join(" ");
    if chunks.len() == 0 {
        None
    } else {
        Some(desc.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let lua_code = r#"
local M = {}

-- this is a sample badge

-- it does useful things.. i think
-- but it mostly demonstrates comment syntax

-- hello there! more comment...


-- @module M
-- @classmod M
-- @author Someone Special
-- @license RC
-- @copyright 2030
-- @release 0.123.v789patch75.rc.alpha3.beta


--- this is a function
-- it does things
-- @function build_badge
-- @param junk this is some junk
-- @tparam string label Label argument
-- @tparam int border_width with of border in pixels
function M:build_badge(inputs)
  return { { "hi" } }
end

--- this is a function to be used internally to the module
-- @function something_internal
-- @param other idk
function M:something_internal(other)
	 return { { "don't parse me, don't eval me"}}
end

return M
"#;
        let result = parse_lua_docs(lua_code);
        // Should error if errored
        result.unwrap();
    }

    #[test]
    fn test_parse_error_missing_param_type() {
        let lua_code = r#"
--- this is a function
-- it does things
-- @function build_badge
-- @param junk this is some junk
-- @tparam blah
-- @tparam int border_width with of border in pixels
function build_badge(inputs)
  return { { "hi" } }
end

return M
"#;
        let result = parse_lua_docs(lua_code);
        // Should error if didn't error
        result.err().unwrap();
    }
}
