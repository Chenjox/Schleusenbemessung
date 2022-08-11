use serde::Deserialize;
use std::fs;

struct Schleusenwerte {
    oberwasser: f64,
    oberwassersohle: f64,
    unterwasser: f64,
    unterwassersohle: f64,
    kammerbreite: f64,
    kammerlaenge: f64,
}

impl Schleusenwerte {
    fn wasserspiegel_oberwasser(&self) -> f64 {
        self.oberwasser - self.oberwassersohle
    }

    fn wasserspiegel_unterwasser(&self) -> f64 {
        self.oberwasser - self.unterwassersohle
    }

    fn hubhoehe(&self) -> f64 {
        self.oberwasser - self.unterwasser
    }

    fn grundflaeche(&self) -> f64 {
        self.kammerbreite * self.kammerlaenge
    }

    fn wasservolumen(&self) -> f64 {
        self.kammerbreite * self.kammerlaenge * (self.hubhoehe() + self.unterwassersohle)
    }
}

#[derive(Deserialize)]
struct Schleuse {
    unterwasser: f64,
    unterwassersohle: f64,
    oberwasser: f64,
    oberwassersohle: f64,
    kammerbreite: f64,
    kammerlaenge: f64,
}

impl From<Schleuse> for Schleusenwerte {
    fn from(s: Schleuse) -> Self {
        Schleusenwerte {
            oberwasser: s.oberwasser,
            oberwassersohle: s.oberwassersohle,
            unterwasser: s.unterwasser,
            unterwassersohle: s.unterwassersohle,
            kammerbreite: s.kammerbreite,
            kammerlaenge: s.kammerlaenge,
        }
    }
}

fn read_schleusenwerte(file_name: &str) -> Result<Schleusenwerte, toml::de::Error> {
    let contents = fs::read_to_string(file_name).unwrap();
    let contents: Result<Schleuse, _> = toml::from_str(&contents);
    let contents: Result<Schleusenwerte, _> = contents.map(|v| Schleusenwerte::from(v));
    return contents;
}

fn main() {
    let schleuse = match read_schleusenwerte("test.toml") {
        Ok(s) => s,
        Err(s) => panic!("{:?}", s),
    };

    let y = schleuse.hubhoehe();
    println!("y = {}", y);
}
