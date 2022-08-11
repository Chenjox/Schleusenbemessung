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

struct FuellRechteck {
    oeffnungsgeschwindigkeit: f64, // in m/s
    breite: f64,
    hoehe: f64,
}

trait Fuellquerschnitt {
    // Fläche des geöffneten Querschnitts zu einem Zeitpunkt s
    fn querschnitt(&self, zeit: f64) -> f64;

    // Ob der Fülllquerschnitt vollständig geöffnet ist..
    fn is_fully_opened(&self, zeit: f64) -> bool;
}

impl Fuellquerschnitt for FuellRechteck {
    fn querschnitt(&self, zeit: f64) -> f64 {
        let temp = zeit * self.oeffnungsgeschwindigkeit;
        return if temp > self.hoehe {
            self.breite * self.hoehe
        } else {
            self.breite * temp
        };
    }

    fn is_fully_opened(&self, zeit: f64) -> bool {
        return zeit * self.oeffnungsgeschwindigkeit > self.hoehe;
    }
}

impl Schleusenwerte {
    fn wasserspiegel_oberwasser(&self) -> f64 {
        self.oberwasser - self.oberwassersohle
    }

    fn wasserspiegel_unterwasser(&self) -> f64 {
        self.unterwasser - self.unterwassersohle
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

fn fuell_schleuse(schl: &Schleusenwerte, quer: &impl Fuellquerschnitt) {
    let mut kammerspiegel: f64 = schl.wasserspiegel_unterwasser();
    let kammerspiegel_max = schl.hubhoehe() + schl.wasserspiegel_unterwasser();
    let mut hydraulische_hoehe: f64 = schl.hubhoehe();

    let leistungsbeiwert: f64 = 0.55;
    let verlustbeiwert: f64 = 0.8;
    let mut durchfluss: f64 = 0.0;

    let zeitschritt = 10.0;
    let mut i = 1;
    let mut volume = schl.grundflaeche() * kammerspiegel;
    let max_iterations = 100;

    while kammerspiegel < kammerspiegel_max && i < max_iterations {
        kammerspiegel = volume / schl.grundflaeche();
        // Der momentane Durchfluss
        durchfluss =
            (2.0 * 9.81 * (schl.hubhoehe() - kammerspiegel + schl.wasserspiegel_unterwasser())
                / (1.0 + verlustbeiwert))
                .sqrt()
                * leistungsbeiwert
                * quer.querschnitt(zeitschritt * (i as f64));
        durchfluss = if durchfluss.is_nan() { 0.0 } else { durchfluss };
        // Ergo der Zuwachs an Volumen ist
        volume += durchfluss * zeitschritt;

        println!(
            "Q = {} m^3/s, t = {} s, K = {} m, V = {} m^3",
            durchfluss,
            zeitschritt * f64::from(i),
            kammerspiegel,
            volume
        );
        // Und die neue hoehe ist...

        i += 1;
    }
}

fn main() {
    let schleuse = match read_schleusenwerte("test.toml") {
        Ok(s) => s,
        Err(s) => panic!("{:?}", s),
    };

    let y = schleuse.hubhoehe();
    println!("y = {}", y);

    let qs = FuellRechteck {
        oeffnungsgeschwindigkeit: 0.005,
        breite: 5.85,
        hoehe: 3.0,
    };

    fuell_schleuse(&schleuse, &qs);
}
