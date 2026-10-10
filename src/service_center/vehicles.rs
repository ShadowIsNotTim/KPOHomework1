use std::rc::Rc;
use ratatui_textarea::TextArea;
use serde::{Serialize, Deserialize};

use super::{AreaVec, ServiceCenter, KWh, FormatError, model_text, parse_f32, parse_bool};
use super::super::{BuildAreasFnT, DoneFnT};
use strum::{Display, EnumIter};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mm(pub f32);

#[typetag::serde]
pub trait Vehicle {
    // интерфейс для транспортных средств
    fn get_id(&self) -> u32;
    fn name(&self) -> String;
    fn get_energy(&self) -> KWh;
    fn get_capacity(&self) -> KWh;

    // Киловатт-часы (KWh) (K большая тк codestyle)
    // увы в тз киловатты именно
    // по этому пришлось флоаты узать

    fn get_consumption(&self) -> KWh {
        KWh(0.0)
    }
    fn simplicity(&self) -> u8 {
        0
    }
    fn for_beginners(&self) -> bool {
        self.simplicity() >= 6
    }
    fn get_model(&self) -> String;
    fn to_string(&self) -> String {
        return format!("{} id-{}", self.get_model(), self.get_id());
    }
    fn build_areas(areas: &mut AreaVec) where Self: Sized;
    fn done(areas: &AreaVec, sc: &mut ServiceCenter) -> Result<(), FormatError> where Self: Sized;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElectricScooter {
    id: u32,
    model: String,
    energy: KWh,
    capacity: KWh,
    wheel_diameter: Mm
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElectricBike {
    id: u32,
    model: String,
    energy: KWh,
    capacity: KWh,
    wheel_diameter: Mm,
    has_passenger_seat: bool
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bicycle {
    id: u32,
    model: String,
    wheel_diameter: Mm
}

#[derive(Debug, Clone, Copy, PartialEq, Display, EnumIter)]
pub enum VehicleTypes {
    #[strum(to_string = "электрический скутер")]
    ElectricScooter,
    #[strum(to_string = "электрический велик")]
    ElectricBike,
    #[strum(to_string = "простой велосипед")]
    Bicycle
}

impl VehicleTypes {
    pub fn unpack(self) -> (String, BuildAreasFnT, DoneFnT) {
        match self {
            VehicleTypes::ElectricScooter => (
                "Новый скутер".to_string(), ElectricScooter::build_areas, ElectricScooter::done
            ),
            VehicleTypes::ElectricBike => (
                "Новый байк".to_string(), ElectricBike::build_areas, ElectricBike::done
            ),
            VehicleTypes::Bicycle => (
                "Новый велосипед".to_string(), Bicycle::build_areas, Bicycle::done
            )
        }
    }
}

#[typetag::serde]
impl Vehicle for ElectricScooter {
    fn get_id(&self) -> u32 {
        self.id
    }

    fn name(&self) -> String {
        format!("{} id-{}", self.model, self.id)
    }

    fn get_energy(&self) -> KWh {
        self.energy
    }

    fn get_capacity(&self) -> KWh {
        self.capacity
    }

    fn get_model(&self) -> String {
        self.model.clone()
    }

    fn to_string(&self) -> String {
        format!(
            "Самокат {} #{}: {}/{} kWh, колесо {} мм, простота {}",
            self.model, self.id, self.energy.0, self.capacity.0,
            self.wheel_diameter.0, self.simplicity()
        )
    }

    fn build_areas(areas: &mut AreaVec) {
        areas.push(("Модель".to_string(), TextArea::default()));
        areas.push(("Заряд (kWh)".to_string(), TextArea::default()));
        areas.push(("Ёмкость (kWh)".to_string(), TextArea::default()));
        areas.push(("Диаметр колеса (мм)".to_string(), TextArea::default()));
    }

    fn done(areas: &AreaVec, sc: &mut ServiceCenter) -> Result<(), FormatError> {
        let id = sc.next_transport_id();
        sc.add_transport(Rc::new(ElectricScooter {
            id,
            model: model_text(areas, 0)?,
            energy: KWh(parse_f32(areas, 1)?),
            capacity: KWh(parse_f32(areas, 2)?),
            wheel_diameter: Mm(parse_f32(areas, 3)?),
        }));
        Ok(())
    }
}

#[typetag::serde]
impl Vehicle for ElectricBike {
    fn get_id(&self) -> u32 {
        self.id
    }

    fn name(&self) -> String {
        format!("{} id-{}", self.model, self.id)
    }

    fn get_energy(&self) -> KWh {
        self.energy
    }

    fn get_capacity(&self) -> KWh {
        self.capacity
    }

    fn get_model(&self) -> String {
        self.model.clone()
    }

    fn to_string(&self) -> String {
        format!(
            "Байк {} #{}: {}/{} kWh, колесо {} мм, пассажир: {}, простота {}",
            self.model, self.id, self.energy.0, self.capacity.0, self.wheel_diameter.0,
            if self.has_passenger_seat { "да" } else { "нет" }, self.simplicity()
        )
    }

    fn build_areas(areas: &mut AreaVec) {
        areas.push(("Модель".to_string(), TextArea::default()));
        areas.push(("Заряд (kWh)".to_string(), TextArea::default()));
        areas.push(("Ёмкость (kWh)".to_string(), TextArea::default()));
        areas.push(("Диаметр колеса (мм)".to_string(), TextArea::default()));
        areas.push(("Пассажирское место? (y/n)".to_string(), TextArea::default()));
    }

    fn done(areas: &AreaVec, sc: &mut ServiceCenter) -> Result<(), FormatError> {
        let id = sc.next_transport_id();
        sc.add_transport(Rc::new(ElectricBike {
            id,
            model: model_text(areas, 0)?,
            energy: KWh(parse_f32(areas, 1)?),
            capacity: KWh(parse_f32(areas, 2)?),
            wheel_diameter: Mm(parse_f32(areas, 3)?),
            has_passenger_seat: parse_bool(areas, 4)?,
        }));
        Ok(())
    }
}

#[typetag::serde]
impl Vehicle for Bicycle {
    fn get_id(&self) -> u32 {
        self.id
    }

    fn name(&self) -> String {
        format!("{} id-{}", self.model, self.id)
    }

    fn get_energy(&self) -> KWh {
        KWh(0.0)
    }

    fn get_capacity(&self) -> KWh {
        KWh(0.0)
    }

    fn simplicity(&self) -> u8 {
        7
    }

    fn get_model(&self) -> String {
        self.model.clone()
    }

    fn to_string(&self) -> String {
        format!(
            "Велосипед {} #{}: колесо {} мм, простота {}",
            self.model, self.id, self.wheel_diameter.0, self.simplicity()
        )
    }

    fn build_areas(areas: &mut AreaVec) {
        areas.push(("Модель".to_string(), TextArea::default()));
        areas.push(("Диаметр колеса (мм)".to_string(), TextArea::default()));
    }

    fn done(areas: &AreaVec, sc: &mut ServiceCenter) -> Result<(), FormatError> {
        let id = sc.next_transport_id();
        sc.add_transport(Rc::new(Bicycle {
            id,
            model: model_text(areas, 0)?,
            wheel_diameter: Mm(parse_f32(areas, 1)?),
        }));
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn areas(values: &[&str]) -> AreaVec {
        values.iter()
            .map(|v| ("field".to_string(), TextArea::new(vec![v.to_string()])))
            .collect()
    }

    #[test]
    fn test_bycycle() {
        let mut areas = AreaVec::new();
        Bicycle::build_areas(&mut areas);

        assert_eq!(areas.len(), 2);
        assert_eq!(areas[0].0, "Модель".to_string());
    }

    #[test]
    fn electric_scooter_done_ok() {
        let a = areas(&["Xiaomi Mi Pro 2", "0.5", "1.0", "8.5"]);
        let mut sc = ServiceCenter::new();
        assert!(ElectricScooter::done(&a, &mut sc).is_ok());
        let (label, _) = sc.transport_options(0).unwrap();
        assert!(label.contains("Xiaomi Mi Pro 2"), "label={label}");
        assert!(label.contains("0.5/1"), "label={label}");
    }

    #[test]
    fn electric_scooter_rejects_bad_floats() {
        for (energy, capacity, wheel) in [
            ("-1", "1.0", "8.5"),
            ("-0", "1.0", "8.5"),
            ("inf", "1.0", "8.5"),
            ("nan", "1.0", "8.5"),
            ("1e999", "1.0", "8.5"),
            ("abc", "1.0", "8.5"),
            ("0.5", "-1", "8.5"),
            ("0.5", "1.0", "inf"),
            ("0.5", "1.0", "nan"),
        ] {
            let a = areas(&["Xiaomi", energy, capacity, wheel]);
            let mut sc = ServiceCenter::new();
            assert!(ElectricScooter::done(&a, &mut sc).is_err(),
                "energy={energy} capacity={capacity} wheel={wheel}");
        }
    }

    #[test]
    fn electric_scooter_rejects_long_and_empty_model() {
        let long = "a".repeat(100);
        let a = areas(&[long.as_str(), "0.5", "1.0", "8.5"]);
        let mut sc = ServiceCenter::new();
        assert!(ElectricScooter::done(&a, &mut sc).is_err());

        let empty = areas(&["", "0.5", "1.0", "8.5"]);
        let mut sc = ServiceCenter::new();
        assert!(ElectricScooter::done(&empty, &mut sc).is_err());
    }

    #[test]
    fn electric_bike_done_ok() {
        let a = areas(&["Super73", "0.9", "1.8", "20", "yes"]);
        let mut sc = ServiceCenter::new();
        assert!(ElectricBike::done(&a, &mut sc).is_ok());
        let (label, _) = sc.transport_options(0).unwrap();
        assert!(label.contains("Super73"), "label={label}");
        assert!(label.contains("пассажир: да"), "label={label}");
    }

    #[test]
    fn electric_bike_rejects_bad_floats() {
        for bad in ["-1", "-0", "inf", "nan", "abc"] {
            let a = areas(&["Super73", bad, "1.8", "20", "yes"]);
            let mut sc = ServiceCenter::new();
            assert!(ElectricBike::done(&a, &mut sc).is_err(), "bad={bad}");

            let a = areas(&["Super73", "0.9", bad, "20", "yes"]);
            let mut sc = ServiceCenter::new();
            assert!(ElectricBike::done(&a, &mut sc).is_err(), "bad={bad}");

            let a = areas(&["Super73", "0.9", "1.8", bad, "yes"]);
            let mut sc = ServiceCenter::new();
            assert!(ElectricBike::done(&a, &mut sc).is_err(), "bad={bad}");
        }
    }

    #[test]
    fn electric_bike_rejects_long_model() {
        let long = "a".repeat(100);
        let a = areas(&[long.as_str(), "0.9", "1.8", "20", "yes"]);
        let mut sc = ServiceCenter::new();
        assert!(ElectricBike::done(&a, &mut sc).is_err());
    }

    #[test]
    fn bicycle_done_ok() {
        let a = areas(&["Stels", "26"]);
        let mut sc = ServiceCenter::new();
        assert!(Bicycle::done(&a, &mut sc).is_ok());
        let (label, _) = sc.transport_options(0).unwrap();
        assert!(label.contains("Stels"), "label={label}");
    }

    #[test]
    fn bicycle_rejects_bad_wheel_and_model() {
        for bad in ["-1", "-0", "inf", "nan", "abc"] {
            let a = areas(&["Stels", bad]);
            let mut sc = ServiceCenter::new();
            assert!(Bicycle::done(&a, &mut sc).is_err(), "bad={bad}");
        }
        let long = "a".repeat(100);
        let a = areas(&[long.as_str(), "26"]);
        let mut sc = ServiceCenter::new();
        assert!(Bicycle::done(&a, &mut sc).is_err());
    }
}