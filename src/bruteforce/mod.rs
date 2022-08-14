use crate::hydraulic::*;

pub struct FuellRechteck {
    pub oeffnungsgeschwindigkeit: f64, // in m/s
    pub breite: f64,
    pub hoehe: f64,
}

fn dhyd(area: f64, umfang: f64) -> f64 {
    return 4.0 * area / umfang;
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

    fn freigegebene_hoehe(&self, zeit: f64) -> f64 {
        return (zeit * self.oeffnungsgeschwindigkeit).min(self.hoehe);
    }

    fn freigegebene_breite(&self, _hoehe: f64) -> f64 {
        return self.breite;
    }

    fn is_fully_opened(&self, zeit: f64) -> bool {
        return zeit * self.oeffnungsgeschwindigkeit > self.hoehe;
    }

    fn durchflussverslust_ueberfall(
        &self,
        _schleuse: &Schleuse,
        _pot_hoehe: f64,
        unterehoehe: f64,
        zeit: f64,
    ) -> f64 {
        //Äquivalente QS Breite
        let b = self.querschnitt(zeit) / (self.freigegebene_hoehe(zeit) - unterehoehe);
        let x = (self.freigegebene_hoehe(zeit) - unterehoehe) / b;
        return 0.673 + x * (-0.0511667 + x * (-0.0105 + x * (-0.047333 + x * (0.018))));
    }

    fn durchflussverslust_unterstroemung(
        &self,
        schleuse: &Schleuse,
        _pot_hoehe: f64,
        unterehoehe: f64,
        zeit: f64,
    ) -> f64 {
        //Einfluss
        let z1: f64 = 0.5;
        //Verengung
        let areafull = self.breite * self.hoehe;
        let areafree = self.breite * self.freigegebene_hoehe(zeit);
        let z2 = 0.5
            * (1.0
                - dhyd(areafull, 2.0 * (self.breite + self.hoehe))
                    / dhyd(
                        areafree,
                        2.0 * (self.breite * self.freigegebene_hoehe(zeit)),
                    ))
            .powi(2);
        // Ausweitung
        let kammerwasserspiegel =
            unterehoehe + schleuse.oberhaupt.oberwassersohle - schleuse.unterhaupt.unterwassersohle;
        let z3 = 1.2
            * (1.0
                - dhyd(
                    areafree,
                    2.0 * (self.breite * self.freigegebene_hoehe(zeit)),
                ) / dhyd(
                    schleuse.kammer.breite * kammerwasserspiegel,
                    (schleuse.kammer.breite) + 2.0 * kammerwasserspiegel,
                ))
            .powi(2);
        return (1.0) / ((1.0 + z1 + z2.max(0.0) + z3.max(0.0)).sqrt());
    }
} // I = 0.018 x^4 + -0.047333 x^3 - 0.0105 x^2 - 0.0511667 x^1 + 0.673
