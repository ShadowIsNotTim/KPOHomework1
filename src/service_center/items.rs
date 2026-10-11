use std::rc::Rc;
use ratatui_textarea::TextArea;
use serde::{Serialize, Deserialize};

use super::{AreaVec, ServiceCenter, Wh, FormatError, required_text, parse_u8, parse_f32, parse_bool};
use super::super::{BuildAreasFnT, DoneFnT};
use strum::{Display, EnumIter};

const MAX_PORTS: u8 = 32;
const MAX_CHARGE_SPEED_WH: f32 = 1000.0;

#[typetag::serde]
pub trait Item {
    fn get_id(&self) -> u32;
    fn name(&self) -> String;
    fn to_string(&self) -> String {
        format!("Item (id={})", self.get_id())
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
    fn set_taken(self: Rc<Self>, taken: bool) -> Rc<dyn Item>;
    fn build_edit_areas(&self, _areas: &mut AreaVec) {}
    fn done_edit(&self, _areas: &AreaVec) -> Result<Rc<dyn Item>, FormatError> {
        Err(FormatError::new("объект не редактируется"))
    }
    fn build_areas(areas: &mut AreaVec) where Self: Sized;
    fn done(areas: &AreaVec, sc: &mut ServiceCenter) -> Result<(), FormatError> where Self: Sized; // добавляем предмет
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChargingStation {
    id: u32,
    port_count: u8,
    charge_speed: Wh,
    taken: bool
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum ClothingSize {
    S,
    M,
    L,
    XL
}

impl ClothingSize {
    fn parse(s: &str) -> Result<Self, FormatError> {
        match s.trim().to_uppercase().as_str() {
            "S" => Ok(ClothingSize::S),
            "M" => Ok(ClothingSize::M),
            "L" => Ok(ClothingSize::L),
            "XL" => Ok(ClothingSize::XL),
            _ => Err(FormatError::new(format!(
                "размер «{}»: ожидалось S/M/L/XL", s.trim()
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Helmet {
    id: u32,
    size: ClothingSize,
    is_damaged: bool,
    taken: bool
}

#[derive(Debug, Clone, Copy, PartialEq, Display, EnumIter)]
pub enum ItemTypes {
    #[strum(to_string = "зарядная станция")]
    ChargingStation,
    #[strum(to_string = "шлем")]
    Helmet
}

impl ItemTypes {
    pub fn unpack(self) -> (String, BuildAreasFnT, DoneFnT) {
        match self {
            ItemTypes::ChargingStation => (
                "Новая зарядная станция".to_string(),
                ChargingStation::build_areas,
                ChargingStation::done
            ),
            ItemTypes::Helmet => (
                "Новый шлем".to_string(),
                Helmet::build_areas,
                Helmet::done
            )
        }
    }
}

#[typetag::serde]
impl Item for ChargingStation {
    fn get_id(&self) -> u32 {
        self.id
    }

    fn name(&self) -> String {
        let base = format!("Зарядная станция #{}", self.id);
        if self.is_taken() { format!("{} (занято)", base) } else { base }
    }

    fn is_taken(&self) -> bool {
        self.taken
    }

    fn set_taken(self: Rc<Self>, taken: bool) -> Rc<dyn Item> {
        let mut i = (*self).clone();
        i.taken = taken;
        Rc::new(i)
    }

    fn to_string(&self) -> String {
        let mut s = format!(
            "Тип: зарядная станция\nID: {}\nКол-во портов: {}\nСкорость зарядки: {} Wh",
            self.id, self.port_count, self.charge_speed.0
        );
        if self.is_taken() {
            s.push_str("\nЗанято");
        }
        s
    }

    fn techoverview(&self) -> Result<(), FormatError> {
        if self.port_count > MAX_PORTS {
            return Err(FormatError::new(format!("порты > {}", MAX_PORTS)));
        }
        if self.charge_speed.0 > MAX_CHARGE_SPEED_WH {
            return Err(FormatError::new(format!("скорость > {} Wh", MAX_CHARGE_SPEED_WH)));
        }
        Ok(())
    }

    fn is_editable(&self) -> bool {
        true
    }

    fn build_edit_areas(&self, areas: &mut AreaVec) {
        areas.push(("Кол-во портов".to_string(), TextArea::new(vec![self.port_count.to_string()])));
        areas.push(("Скорость зарядки (Wh)".to_string(), TextArea::new(vec![self.charge_speed.0.to_string()])));
    }

    fn done_edit(&self, areas: &AreaVec) -> Result<Rc<dyn Item>, FormatError> {
        let port_count = parse_u8(areas, 0)?;
        if port_count == 0 {
            return Err(FormatError::new(format!(
                "поле «{}»: должно быть хотя бы 1", areas[0].0
            )));
        }
        let charge_speed = parse_f32(areas, 1)?;
        Ok(Rc::new(ChargingStation {
            id: self.id,
            port_count,
            charge_speed: Wh(charge_speed),
            taken: self.taken,
        }))
    }

    fn build_areas(areas: &mut AreaVec) {
        areas.push(("Кол-во портов".to_string(), TextArea::default()));
        areas.push(("Скорость зарядки (Wh)".to_string(), TextArea::default()));
    }

    fn done(areas: &AreaVec, sc: &mut ServiceCenter) -> Result<(), FormatError> {
        let port_count = parse_u8(areas, 0)?;
        if port_count == 0 {
            return Err(FormatError::new(format!(
                "поле «{}»: должно быть хотя бы 1", areas[0].0
            )));
        }
        let charge_speed = parse_f32(areas, 1)?;
        let id = sc.next_item_id();
        sc.add_item(Rc::new(ChargingStation {
            id,
            port_count,
            charge_speed: Wh(charge_speed),
            taken: false,
        }));
        Ok(())
    }
}

#[typetag::serde]
impl Item for Helmet {
    fn get_id(&self) -> u32 {
        self.id
    }

    fn name(&self) -> String {
        let base = format!("Шлем #{}", self.id);
        if self.is_taken() { format!("{} (занято)", base) } else { base }
    }

    fn is_taken(&self) -> bool {
        self.taken
    }

    fn set_taken(self: Rc<Self>, taken: bool) -> Rc<dyn Item> {
        let mut i = (*self).clone();
        i.taken = taken;
        Rc::new(i)
    }

    fn to_string(&self) -> String {
        let mut s = format!(
            "Тип: шлем\nID: {}\nРазмер: {:?}\nПовреждён: {}",
            self.id,
            self.size,
            if self.is_damaged { "да" } else { "нет" }
        );
        if self.is_taken() {
            s.push_str("\nЗанято");
        }
        s
    }

    fn techoverview(&self) -> Result<(), FormatError> {
        Ok(())
    }

    fn build_areas(areas: &mut AreaVec) {
        areas.push(("Размер (S/M/L/XL)".to_string(), TextArea::default()));
        areas.push(("Повреждён? (y/n)".to_string(), TextArea::default()));
    }

    fn done(areas: &AreaVec, sc: &mut ServiceCenter) -> Result<(), FormatError> {
        let size = ClothingSize::parse(&required_text(areas, 0)?)?;
        let is_damaged = parse_bool(areas, 1)?;
        let id = sc.next_item_id();
        sc.add_item(Rc::new(Helmet { id, size, is_damaged, taken: false }));
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
    fn clothing_size_parse_ok() {
        let cases = [
            ("S", ClothingSize::S),
            ("s", ClothingSize::S),
            ("M", ClothingSize::M),
            ("m", ClothingSize::M),
            ("L", ClothingSize::L),
            ("l", ClothingSize::L),
            ("XL", ClothingSize::XL),
            ("xl", ClothingSize::XL),
            (" Xl ", ClothingSize::XL),
        ];
        for (raw, expected) in cases {
            assert_eq!(ClothingSize::parse(raw).unwrap(), expected, "raw={raw}");
        }
    }

    #[test]
    fn clothing_size_parse_rejects_wrong_layout() {
        for raw in ["М", "м", "Ь", "ь", "Ы", "ы", "Д", "д", "Х", "х", "Ч", "ч", "ЧД", "чд", "хl", "Хl"] {
            assert!(ClothingSize::parse(raw).is_err(), "raw={raw}");
        }
    }

    #[test]
    fn clothing_size_parse_invalid() {
        for raw in ["", "XXL", "1", "ф", "SML", "xs", "X"] {
            assert!(ClothingSize::parse(raw).is_err(), "raw={raw}");
        }
    }

    #[test]
    fn charging_station_done_ok() {
        let a = areas(&["2", "22.5"]);
        let mut sc = ServiceCenter::new();
        assert!(ChargingStation::done(&a, &mut sc).is_ok());
        assert!(sc.item_options(0).is_some());
    }

    #[test]
    fn charging_station_done_rejects_negative_speed() {
        for bad in ["-1", "-0", "-0.0"] {
            let a = areas(&["2", bad]);
            let mut sc = ServiceCenter::new();
            assert!(ChargingStation::done(&a, &mut sc).is_err(), "bad={bad}");
        }
    }

    #[test]
    fn charging_station_done_rejects_infinite_speed() {
        for bad in ["inf", "-inf", "1e999"] {
            let a = areas(&["2", bad]);
            let mut sc = ServiceCenter::new();
            assert!(ChargingStation::done(&a, &mut sc).is_err(), "bad={bad}");
        }
    }

    #[test]
    fn charging_station_done_rejects_nan_speed() {
        let a = areas(&["2", "nan"]);
        let mut sc = ServiceCenter::new();
        assert!(ChargingStation::done(&a, &mut sc).is_err());
    }

    #[test]
    fn charging_station_done_rejects_bad_port_count() {
        for bad in ["0", "300", "-1", "x"] {
            let a = areas(&[bad, "22"]);
            let mut sc = ServiceCenter::new();
            assert!(ChargingStation::done(&a, &mut sc).is_err(), "bad={bad}");
        }
    }

    #[test]
    fn helmet_done_ok() {
        let a = areas(&["M", "yes"]);
        let mut sc = ServiceCenter::new();
        assert!(Helmet::done(&a, &mut sc).is_ok());
        assert!(sc.item_options(0).is_some());
    }

    #[test]
    fn helmet_done_rejects_bad_size_and_flag() {
        let bad_size = areas(&["Гигантский", "no"]);
        let mut sc = ServiceCenter::new();
        assert!(Helmet::done(&bad_size, &mut sc).is_err());

        let bad_flag = areas(&["L", "maybe"]);
        let mut sc = ServiceCenter::new();
        assert!(Helmet::done(&bad_flag, &mut sc).is_err());
    }

    #[test]
    fn charging_station_techoverview() {
        let ok = ChargingStation { id: 0, port_count: 2, charge_speed: Wh(22.0), taken: false };
        assert!(ok.techoverview().is_ok());

        let many_ports = ChargingStation { id: 0, port_count: 64, charge_speed: Wh(22.0), taken: false };
        assert!(many_ports.techoverview().is_err());

        let fast = ChargingStation { id: 0, port_count: 2, charge_speed: Wh(5000.0), taken: false };
        assert!(fast.techoverview().is_err());
    }

    #[test]
    fn helmet_techoverview() {
        let h = Helmet { id: 0, size: ClothingSize::M, is_damaged: true, taken: false };
        assert!(h.techoverview().is_ok());
    }

    #[test]
    fn charging_station_edit_keeps_id() {
        let orig = ChargingStation { id: 3, port_count: 2, charge_speed: Wh(22.0), taken: false };
        assert!(orig.is_editable());
        let a = areas(&["4", "50.0"]);
        let updated = orig.done_edit(&a).unwrap();
        assert_eq!(updated.get_id(), 3);
    }

    #[test]
    fn helmet_is_not_editable() {
        let h = Helmet { id: 0, size: ClothingSize::M, is_damaged: false, taken: false };
        assert!(!h.is_editable());
        let a = areas(&["L", "no"]);
        assert!(h.done_edit(&a).is_err());
    }
}

