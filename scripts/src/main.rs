use std::{fs::{self, File}, time::Instant, io::{BufWriter, Write}, collections::HashSet};
use xml::{attribute::OwnedAttribute, reader::{self, XmlEvent}};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Entry {
    word: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    selmaho: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    rafsi: Vec<String>,
    score: i32,
    definition: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    notes: String,
    #[serde(skip)]
    pos: String,
    #[serde(skip)]
    author: String,
}
impl Entry {
    fn new() -> Self {
        Self {
            word: String::new(),
            rafsi: Vec::new(),
            selmaho: String::new(),
            score: 0,
            definition: String::new(),
            notes: String::new(),
            pos: String::new(),
            author: String::new()
        }
    }
    fn to_datastring(self) -> String {
        let mut s = self.word.replace(" ", "_").replace(".", "_").replace(r"^_|_$", "").replace(r"_+", "_");
        s += &(" ".to_owned() + &self.pos);
        if self.selmaho.len() > 0 {
            s += &(" ".to_owned() + &self.selmaho);
        }
        if self.rafsi.len() > 0 {
            s += &(" [-".to_owned() + &self.rafsi.join("-") + "-]");
        }
        s += &(" ".to_owned() + &self.author.to_lowercase().replace(r"[^a-z0-9]", ""));
        s += &(" ".to_owned() + self.score.to_string().as_str());
        s += &("\n".to_owned() + &self.definition);
        if self.notes.len() > 0 {
            s += &("\n-n\n".to_owned() + &self.notes);
        }
        s
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    // parse the xml
    let langs = ["en", "am", "ar", "art-guaspi", "art-loglan", "be", "bg", "br", "ca", "ch", "cs", "cy", "da", "de", "el", "en-bpfk", "en-simple", "eo", "es", "et", "eu", "fa", "fi", "fr-facile", "fr", "ga", "gl", "gu", "he", "hi", "hr", "hu", "ia", "id", "it", "ja", "jbo", "ka", "ko", "kw", "la", "lt", "lv", "mg", "ne", "nl", "no", "pl", "pt-br", "pt", "ro", "ru", "sa", "sk", "sl", "so", "sq", "sr", "sv", "ta", "test", "tlh", "tok", "tpi", "tr", "uk", "vi", "wa", "zh"];
    let mut words = Vec::<Entry>::new();
    let mut current_tag = String::new();
    let mut entry = Entry::new();
    let mut skip = false;
    for lang in langs {
        let f = fs::File::open(format!("../jvs/jbovlaste-{lang}.xml"))?;
        let mut reader = reader::EventReader::new(f); // we don't need the file anymore
        println!("parsing `{lang}`...");
        loop {
            match reader.next()? {
                XmlEvent::EndDocument { .. } => {
                    break;
                }
                XmlEvent::StartElement { name, attributes, .. } => {
                    let tagname = name.local_name;
                    match tagname.as_str() {
                        "valsi" => {
                            entry = Entry::new();
                            if !attr(&attributes, "type").starts_with('o') {
                                entry.word = attr(&attributes, "word");
                                entry.pos = attr(&attributes, "type");
                                skip = false;
                            } else {
                                current_tag.clear();
                                reader.skip()?;
                                skip = true;
                            }
                        }
                        "score" | "rafsi" | "selmaho" | "definition" | "notes" | "username" => {
                            current_tag = tagname;
                        }
                        "dictionary" | "direction" | "user" => {
                            // go inside
                        }
                        _ => {
                            reader.skip()?;
                        }
                    }
                }
                XmlEvent::Characters(text) => {
                    match current_tag.as_str() {
                        "score" => {
                            let int = text.parse::<i32>().unwrap();
                            if int >= -1 {
                                entry.score = int;
                            } else {
                                skip = true;
                            }
                        }
                        "rafsi" => {
                            entry.rafsi.push(text);
                        }
                        "selmaho" => {
                            entry.selmaho = text;
                        }
                        "definition" => {
                            entry.definition = text;
                        }
                        "notes" => {
                            entry.notes = text;
                        }
                        "username" => {
                            entry.author = text;
                        }
                        _ => ()
                    }
                    current_tag.clear();
                }
                XmlEvent::EndElement { name } => {
                    let tagname = name.local_name;
                    if tagname == "valsi" && !skip {
                        words.push(entry.clone());
                    }
                }
                _ => ()
            }
        }
    }
    // remove duplicates
    let mut unique_words = HashSet::new();
    words.retain(|word| unique_words.insert(word.word.clone()));
    // write to json
    println!("writing to json...");
    let json_str = serde_json::to_string(&words)?;
    fs::write("../jbo.json", json_str)?;
    // make data.txt
    println!("writing to data.txt...");
    let mut f = BufWriter::new(File::create("../data.txt")?);
    writeln!(f, "---")?;
    for word in words {
        writeln!(f, "{}\n---", word.to_datastring())?;
    }
    // .i mulno .ui
    let duration = start.elapsed();
    println!("done :3 took {:?} s", duration.as_secs_f64());
    Ok(())
}

fn attr(v: &[OwnedAttribute], n: &str) -> String {
    v.iter().find(|&x| x.name.local_name == n).unwrap().value.to_string()
}