use std::rc::Rc;
use ratatui_textarea::TextArea;
use serde::{Serialize, Deserialize};

use super::{AreaVec, ServiceCenter, KWh, FormatError, model_text, parse_f32, parse_bool};
use super::super::{BuildAreasFnT, DoneFnT};
use strum::{Display, EnumIter};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mm(pub f32);

const MAX_WHEEL_MM: f32 = 1000.0;
const MAX_CAPACITY_KWH: f32 = 100.0;

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
    fn techoverview(&self) -> Result<(), FormatError>;
    fn to_string_with_overview(&self) -> String {
        match self.techoverview() {
            Ok(()) => format!("{}\n\nТехосмотр: OK", self.to_string()),
            Err(e) => format!("{}\n\nТехосмотр: ERR — {}", self.to_string(), e),
        }
    }
    fn is_editable(&self) -> bool {
        false
    }
    fn is_taken(&self) -> bool;
    fn set_taken(self: Rc<Self>, taken: bool) -> Rc<dyn Vehicle>;
    fn build_edit_areas(&self, _areas: &mut AreaVec) {}
    fn done_edit(&self, _areas: &AreaVec) -> Result<Rc<dyn Vehicle>, FormatError> {
        Err(FormatError::new("объект не редактируется"))
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
    wheel_diameter: Mm,
    taken: bool
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElectricBike {
    id: u32,
    model: String,
    energy: KWh,
    capacity: KWh,
    wheel_diameter: Mm,
    has_passenger_seat: bool,
    taken: bool
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bicycle {
    id: u32,
    model: String,
    wheel_diameter: Mm,
    taken: bool
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
        let base = format!("{} id-{}", self.model, self.id);
        if self.is_taken() { format!("{} (занято)", base) } else { base }
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

    fn is_taken(&self) -> bool {
        self.taken
    }

    fn set_taken(self: Rc<Self>, taken: bool) -> Rc<dyn Vehicle> {
        let mut v = (*self).clone();
        v.taken = taken;
        Rc::new(v)
    }

    fn to_string(&self) -> String {
        let mut s = format!(
            "Тип: электросамокат\nМодель: {}\nID: {}\nЗаряд: {} kWh\nЁмкость: {} kWh\nДиаметр колеса: {} мм\nПростота: {}",
            self.model, self.id, self.energy.0, self.capacity.0,
            self.wheel_diameter.0, self.simplicity()
        );
        if self.is_taken() {
            s.push_str("\nЗанято");
        }
        s
    }

    fn techoverview(&self) -> Result<(), FormatError> {
        if self.wheel_diameter.0 > MAX_WHEEL_MM {
            return Err(FormatError::new(format!("колесо > {} мм", MAX_WHEEL_MM)));
        }
        if self.capacity.0 > MAX_CAPACITY_KWH {
            return Err(FormatError::new(format!("ёмкость > {} kWh", MAX_CAPACITY_KWH)));
        }
        if self.energy.0 > self.capacity.0 {
            return Err(FormatError::new("заряд > ёмкости".to_string()));
        }
        Ok(())
    }

    fn is_editable(&self) -> bool {
        true
    }

    fn build_edit_areas(&self, areas: &mut AreaVec) {
        areas.push(("Модель".to_string(), TextArea::new(vec![self.model.clone()])));
        areas.push(("Заряд (kWh)".to_string(), TextArea::new(vec![self.energy.0.to_string()])));
        areas.push(("Ёмкость (kWh)".to_string(), TextArea::new(vec![self.capacity.0.to_string()])));
        areas.push(("Диаметр колеса (мм)".to_string(), TextArea::new(vec![self.wheel_diameter.0.to_string()])));
    }

    fn done_edit(&self, areas: &AreaVec) -> Result<Rc<dyn Vehicle>, FormatError> {
        Ok(Rc::new(ElectricScooter {
            id: self.id,
            model: model_text(areas, 0)?,
            energy: KWh(parse_f32(areas, 1)?),
            capacity: KWh(parse_f32(areas, 2)?),
            wheel_diameter: Mm(parse_f32(areas, 3)?),
            taken: self.taken,
        }))
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
            taken: false,
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
        let base = format!("{} id-{}", self.model, self.id);
        if self.is_taken() { format!("{} (занято)", base) } else { base }
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

    fn is_taken(&self) -> bool {
        self.taken
    }

    fn set_taken(self: Rc<Self>, taken: bool) -> Rc<dyn Vehicle> {
        let mut v = (*self).clone();
        v.taken = taken;
        Rc::new(v)
    }

    fn to_string(&self) -> String {
        let mut s = format!(
            "Тип: электробайк\nМодель: {}\nID: {}\nЗаряд: {} kWh\nЁмкость: {} kWh\nДиаметр колеса: {} мм\nПассажирское место: {}\nПростота: {}",
            self.model, self.id, self.energy.0, self.capacity.0, self.wheel_diameter.0,
            if self.has_passenger_seat { "да" } else { "нет" }, self.simplicity()
        );
        if self.is_taken() {
            s.push_str("\nЗанято");
        }
        s
    }

    fn techoverview(&self) -> Result<(), FormatError> {
        if self.wheel_diameter.0 > MAX_WHEEL_MM {
            return Err(FormatError::new(format!("колесо > {} мм", MAX_WHEEL_MM)));
        }
        if self.capacity.0 > MAX_CAPACITY_KWH {
            return Err(FormatError::new(format!("ёмкость > {} kWh", MAX_CAPACITY_KWH)));
        }
        if self.energy.0 > self.capacity.0 {
            return Err(FormatError::new("заряд > ёмкости".to_string()));
        }
        Ok(())
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
            taken: false,
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
        let base = format!("{} id-{}", self.model, self.id);
        if self.is_taken() { format!("{} (занято)", base) } else { base }
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

    fn is_taken(&self) -> bool {
        self.taken
    }

    fn set_taken(self: Rc<Self>, taken: bool) -> Rc<dyn Vehicle> {
        let mut v = (*self).clone();
        v.taken = taken;
        Rc::new(v)
    }

    fn to_string(&self) -> String {
        let mut s = format!(
            "Тип: велосипед\nМодель: {}\nID: {}\nДиаметр колеса: {} мм\nПростота: {}",
            self.model, self.id, self.wheel_diameter.0, self.simplicity()
        );
        if self.is_taken() {
            s.push_str("\nЗанято");
        }
        s
    }

    fn techoverview(&self) -> Result<(), FormatError> {
        if self.wheel_diameter.0 > MAX_WHEEL_MM {
            return Err(FormatError::new(format!("колесо > {} мм", MAX_WHEEL_MM)));
        }
        Ok(())
    }

    fn is_editable(&self) -> bool {
        true
    }

    fn build_edit_areas(&self, areas: &mut AreaVec) {
        areas.push(("Модель".to_string(), TextArea::new(vec![self.model.clone()])));
        areas.push(("Диаметр колеса (мм)".to_string(), TextArea::new(vec![self.wheel_diameter.0.to_string()])));
    }

    fn done_edit(&self, areas: &AreaVec) -> Result<Rc<dyn Vehicle>, FormatError> {
        Ok(Rc::new(Bicycle {
            id: self.id,
            model: model_text(areas, 0)?,
            wheel_diameter: Mm(parse_f32(areas, 1)?),
            taken: self.taken,
        }))
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
            taken: false,
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
        assert!(sc.transport_options(0).is_some());
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
        assert!(sc.transport_options(0).is_some());
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
        assert!(sc.transport_options(0).is_some());
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

    #[test]
    fn bicycle_techoverview() {
        let ok = Bicycle { id: 0, model: "X".to_string(), wheel_diameter: Mm(26.0), taken: false };
        assert!(ok.techoverview().is_ok());

        let absurd = Bicycle { id: 0, model: "X".to_string(), wheel_diameter: Mm(5000.0), taken: false };
        assert!(absurd.techoverview().is_err());
    }

    #[test]
    fn electric_scooter_techoverview() {
        let ok = ElectricScooter {
            id: 0, model: "X".to_string(), energy: KWh(0.5), capacity: KWh(1.0),
            wheel_diameter: Mm(8.5), taken: false,
        };
        assert!(ok.techoverview().is_ok());

        let big_wheel = ElectricScooter {
            id: 0, model: "X".to_string(), energy: KWh(0.5), capacity: KWh(1.0),
            wheel_diameter: Mm(5000.0), taken: false,
        };
        assert!(big_wheel.techoverview().is_err());

        let big_capacity = ElectricScooter {
            id: 0, model: "X".to_string(), energy: KWh(0.5), capacity: KWh(1000.0),
            wheel_diameter: Mm(8.5), taken: false,
        };
        assert!(big_capacity.techoverview().is_err());

        let impossible = ElectricScooter {
            id: 0, model: "X".to_string(), energy: KWh(5.0), capacity: KWh(1.0),
            wheel_diameter: Mm(8.5), taken: false,
        };
        assert!(impossible.techoverview().is_err());
    }

    #[test]
    fn electric_bike_techoverview() {
        let ok = ElectricBike {
            id: 0, model: "X".to_string(), energy: KWh(0.5), capacity: KWh(1.0),
            wheel_diameter: Mm(20.0), has_passenger_seat: true, taken: false,
        };
        assert!(ok.techoverview().is_ok());

        let absurd = ElectricBike {
            id: 0, model: "X".to_string(), energy: KWh(0.5), capacity: KWh(1.0),
            wheel_diameter: Mm(100000.0), has_passenger_seat: false, taken: false,
        };
        assert!(absurd.techoverview().is_err());
    }

    #[test]
    fn electric_scooter_edit_keeps_id_and_updates_fields() {
        let orig = ElectricScooter {
            id: 7, model: "Old".to_string(), energy: KWh(0.5), capacity: KWh(1.0),
            wheel_diameter: Mm(8.5), taken: false,
        };
        assert!(orig.is_editable());
        let a = areas(&["New", "0.6", "1.2", "9.0"]);
        let updated = orig.done_edit(&a).unwrap();
        assert_eq!(updated.get_id(), 7);
        assert_eq!(updated.get_model(), "New");
    }

    #[test]
    fn electric_bike_is_not_editable() {
        let b = ElectricBike {
            id: 0, model: "X".to_string(), energy: KWh(0.5), capacity: KWh(1.0),
            wheel_diameter: Mm(20.0), has_passenger_seat: false, taken: false,
        };
        assert!(!b.is_editable());
        let a = areas(&["Y", "0.6", "1.2", "21", "no"]);
        assert!(b.done_edit(&a).is_err());
    }

    #[test]
    fn bicycle_edit_keeps_id() {
        let orig = Bicycle { id: 4, model: "Old".to_string(), wheel_diameter: Mm(26.0), taken: false };
        assert!(orig.is_editable());
        let a = areas(&["New", "27"]);
        let updated = orig.done_edit(&a).unwrap();
        assert_eq!(updated.get_id(), 4);
        assert_eq!(updated.get_model(), "New");
    }
}