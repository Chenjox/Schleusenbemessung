const G: f64 = 9.81;

pub trait Fuellquerschnitt {
    // Fläche des geöffneten Querschnitts zu einem Zeitpunkt s
    fn querschnitt(&self, zeit: f64) -> f64;

    // Freigegebene Hohe des Querschnitts
    fn freigegebene_hoehe(&self, zeit: f64) -> f64;

    // Freigegebene Breite bei einer gewissen hoehe des Querschnitts
    fn freigegebene_breite(&self, hoehe: f64) -> f64;

    // Ob der Fülllquerschnitt vollständig geöffnet ist..
    fn is_fully_opened(&self, zeit: f64) -> bool;

    // Quadratur zur Ermittlung des Durchflusses
    // Die Potentialhoehe ist anzugeben auf die untere Kante des Füllquerschnitts
    // Die Überstromte Höhe des Füllquerschnitts ist die obere Kante des Füllquerschnitts
    fn quadratur_durchfluss_unterstroemung(
        &self,
        pot_hoehe: f64,
        uberstroemte_hoehe: f64,
        zeit: f64,
    ) -> f64 {
        let frei_hoehe = self.freigegebene_hoehe(zeit);
        // Welche obere Grenze ist maßgebend?
        let ober = frei_hoehe.min(uberstroemte_hoehe);
        let n: u32 = 100;
        let schritt = (ober) / (n as f64);
        // Trapezformel!
        let unteregrenze =
            self.freigegebene_breite(0.0) * (2.0 * G * (pot_hoehe - uberstroemte_hoehe)).sqrt();
        let oberegrenze =
            self.freigegebene_breite(ober) * (2.0 * G * (pot_hoehe - uberstroemte_hoehe)).sqrt();
        let mut rest = 0.0;
        for i in 1..(n - 1) {
            rest += self.freigegebene_breite(schritt * i as f64)
                * (2.0 * G * (pot_hoehe - (uberstroemte_hoehe))).sqrt();
        }
        return (rest + unteregrenze + oberegrenze) * schritt;
    }

    fn quadratur_durchfluss_ueberfall(
        &self,
        pot_hoehe: f64,
        uberstroemte_hoehe: f64,
        zeit: f64,
    ) -> f64 {
        let frei_hoehe = self.freigegebene_hoehe(zeit);
        if frei_hoehe < uberstroemte_hoehe {
            // Ist der Querschnitt komplett uberfüllt, ist der Uberfall 0
            return 0.0;
        }
        let n = 100;
        let schritt = (frei_hoehe - uberstroemte_hoehe) / (n as f64);

        let unteregrenze = self.freigegebene_breite(uberstroemte_hoehe)
            * (2.0 * G * (pot_hoehe - uberstroemte_hoehe)).sqrt();
        let oberegrenze =
            self.freigegebene_breite(frei_hoehe) * (2.0 * G * (pot_hoehe - frei_hoehe)).sqrt();
        let mut rest = 0.0;
        for i in 1..(n - 1) {
            rest += self.freigegebene_breite(uberstroemte_hoehe + schritt * i as f64)
                * (2.0 * G * (pot_hoehe - (uberstroemte_hoehe + schritt * i as f64))).sqrt();
        }
        return (rest + unteregrenze + oberegrenze) * schritt;
    }
}

pub struct Schleusenkammer {
    pub breite: f64,
    pub laenge: f64,
}

pub struct Oberhaupt {
    pub oberwasser: f64,
    pub oberwassersohle: f64,
}

pub struct Fuellquerschnittssystem {
    pub hoehe: f64, // Unterkante des Querschnitts ab Bezugshöhe
    pub fuellquerschnitt: Box<dyn Fuellquerschnitt>,
}

impl Fuellquerschnittssystem {
    /**
    Berechnet den Durchfluss des Füllsystems abhängig von der Zeit und der Bezugshöhe der Kammer
    Ist Höhe hoch genug ist kombinierter Zufluss anzusetzen
    Die obere hoehe gibt die Höhe des OWs bzw. des Kammerwassers an. (= Potential von dem Ausgegangen wird.)
    Die untere hoehe gibt die Höhe des Kammerwassers bzw des UWs an. (= Ob Querschnitt teilweise rückgestaut ist.)
    Beide Höhen sind ausgehend von der Bezugshöhe angegeben.
    */
    pub fn durchfluss(&self, oberehoehe: f64, unterehoehe: f64, zeit: f64) -> f64 {
        let pot_hoehe = oberehoehe - self.hoehe;

        if unterehoehe < self.hoehe {
            self.fuellquerschnitt
                .quadratur_durchfluss_ueberfall(pot_hoehe, 0.0, zeit)
        } else {
            self.fuellquerschnitt.quadratur_durchfluss_unterstroemung(
                pot_hoehe,
                unterehoehe - self.hoehe,
                zeit,
            ) + self.fuellquerschnitt.quadratur_durchfluss_ueberfall(
                pot_hoehe,
                unterehoehe - self.hoehe,
                zeit,
            )
        }
    }
}

pub struct Fuellsystem {
    pub querschnitte: Vec<Box<Fuellquerschnittssystem>>,
}

pub struct Unterhaupt {
    pub unterwasser: f64,
    pub unterwassersohle: f64,
}
pub struct Schleuse {
    pub kammer: Schleusenkammer,
    pub oberhaupt: Oberhaupt,
    pub unterhaupt: Unterhaupt,
    pub fuellsystem: Fuellsystem,
}

impl Oberhaupt {
    pub fn wasserspiegel(&self) -> f64 {
        self.oberwasser - self.oberwassersohle
    }
}

impl Unterhaupt {
    pub fn wasserspiegel(&self) -> f64 {
        self.unterwasser - self.unterwassersohle
    }
}

impl Schleusenkammer {
    pub fn grundflaeche(&self) -> f64 {
        self.breite * self.laenge
    }
}

impl Fuellsystem {
    fn durchfluss(&self, unterehoehe: f64, oberehoehe: f64, zeit: f64) -> f64 {
        let mut res = 0.0;
        for i in &self.querschnitte {
            res += i.durchfluss(oberehoehe, unterehoehe, zeit);
        }
        return res;
    }
}

impl Schleuse {
    fn hubhoehe(&self) -> f64 {
        self.oberhaupt.oberwasser - self.unterhaupt.unterwasser
    }

    fn wasservolumen(&self) -> f64 {
        self.kammer.grundflaeche() * (self.hubhoehe() + self.unterhaupt.wasserspiegel())
    }

    pub fn fuell_schleuse(&self) -> Vec<[f64; 5]> {
        let mut kammerspiegel = self.unterhaupt.wasserspiegel();
        let zeitschritt = 1.0;
        let mut i = 1;
        let mut volume = self.kammer.grundflaeche() * kammerspiegel;
        let max_iterations = 2000;

        let mut result_vec = Vec::new();
        let mut durchfluss = 0.0;

        while kammerspiegel < self.oberhaupt.oberwasser - self.unterhaupt.unterwassersohle
            && i < max_iterations
        {
            kammerspiegel = volume / self.kammer.grundflaeche();
            let unterehoehe = (kammerspiegel
                - (self.oberhaupt.oberwassersohle - self.unterhaupt.unterwassersohle))
                .max(0.0);
            let oberehoehe = self.oberhaupt.wasserspiegel();
            let durchfluss_alt = durchfluss;
            durchfluss = 0.65
                * self
                    .fuellsystem
                    .durchfluss(unterehoehe, oberehoehe, zeitschritt * (i as f64));
            let durchfluss = if durchfluss.is_nan() { 0.0 } else { durchfluss };
            volume += durchfluss * zeitschritt;

            let wellengeschwindigkeit = (kammerspiegel * G).sqrt();
            let wasserspiegelneigung = (durchfluss - durchfluss_alt)
                / (zeitschritt
                    * self.kammer.breite
                    * wellengeschwindigkeit
                    * wellengeschwindigkeit)
                * 10.0e3;

            result_vec.push([
                zeitschritt * f64::from(i),
                durchfluss,
                kammerspiegel,
                volume,
                wasserspiegelneigung,
            ]);

            i += 1;
        }

        return result_vec;
    }
}
