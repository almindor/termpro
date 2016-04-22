use std::boxed::Box;
use std::fs;
use std::path::Path;
use std::str;

pub struct AbsPathExpander {}

impl Expands for AbsPathExpander {
    fn takes(&self, source_path: &str) -> Result<bool, Error> {
        let raw_path = source_path.split_whitespace().last().unwrap_or("");
        let source = Path::new(raw_path);
        Ok(source.is_absolute())
    }

    fn expand(&mut self, source_path: &str) -> Result<Expansion, Error> {
        let raw_path = source_path.split_whitespace().last().unwrap_or("");
        let source = Path::new(raw_path);
        let mut exp = Expansion::new();
        let mut index = 0;

        let (base_path, source_fn) = match source.exists() {
            false => match source.parent() {
                Some(bp) => (bp, source.file_name().unwrap().to_str().unwrap_or("")),
                None => return Ok(exp)
            },
            true => (source, "")
        };

        if base_path.is_dir() {
            let dr = try!(fs::read_dir(base_path)).filter(|entry| match *entry {
                Ok(ref e) => {
                    e.file_name().to_str().unwrap_or("").starts_with(source_fn)
                },
                Err(_) => false
            } );

            for de in dr {
                match de {
                    Err(err) => return Err(err),
                    Ok(item) => {
                        let fin = item.file_name();
                        let fin_str = fin.to_str().unwrap_or("");
                        if fin_str.len() == 0 {
                            continue
                        }

                        let entry_string = (&exp.entry).to_string();
                        let exp_string = entry_string + &fin_str[source_fn.len() ..];
                        let full_string = String::from(raw_path) + exp_string.as_str();
                        let full_path = Path::new(full_string.as_str());
                        let ending = match full_path.is_dir() {
                            true => "/",
                            false => match full_path.is_file() {
                                true => " ",
                                false => "" // what is it then? :D
                            }
                        };

                        exp.hints.push(String::from(fin_str) + ending);
                        if index == 0 {
                            exp.entry.push_str(exp_string.as_str());
                            exp.entry.push_str(ending);
                        }
                    }
                };
                index += 1;
            }
        }

        Ok(exp)
    }
}
