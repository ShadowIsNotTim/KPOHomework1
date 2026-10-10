use std::rc::Rc;
use ratatui_textarea::TextArea;
use serde::{Serialize, Deserialize};

use super::{AreaVec, ServiceCenter, Wh, FormatError, required_text, parse_u8, parse_f32, parse_bool};
use super::super::{BuildAreasFnT, DoneFnT};
use strum::{Display, EnumIter};


#[typetag::serde]
pub trait Item {
    fn get_id(&self) -> u32;
    fn name(&self) -> String;
    fn to_string(&self) -> String {
        format!("Item (id={})", self.get_id())
    }
    fn build_areas(areas: &mut AreaVec) where Self: Sized;
    fn done(areas: &AreaVec, sc: &mut ServiceCenter) -> Result<(), FormatError> where Self: Sized; // добавляем предмет
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChargingStation {
    id: u32,
    port_count: u8,
    charge_speed: Wh
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
    is_damaged: bool
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
        format!("Зарядная станция #{}", self.id)
    }

    fn to_string(&self) -> String {
        format!(
            "Зарядная станция #{}: портов {}, скорость зарядки {} Wh",
            self.id, self.port_count, self.charge_speed.0
        )
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
        format!("Шлем #{}", self.id)
    }

    fn to_string(&self) -> String {
        format!(
            "Шлем #{}: размер {:?}, повреждён: {}",
            self.id,
            self.size,
            if self.is_damaged { "да" } else { "нет" }
        )
    }

    fn build_areas(areas: &mut AreaVec) {
        areas.push(("Размер (S/M/L/XL)".to_string(), TextArea::default()));
        areas.push(("Повреждён? (y/n)".to_string(), TextArea::default()));
    }

    fn done(areas: &AreaVec, sc: &mut ServiceCenter) -> Result<(), FormatError> {
        let size = ClothingSize::parse(&required_text(areas, 0)?)?;
        let is_damaged = parse_bool(areas, 1)?;
        let id = sc.next_item_id();
        sc.add_item(Rc::new(Helmet { id, size, is_damaged }));
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
        let (label, _) = sc.item_options(0).unwrap();
        assert!(label.contains("портов 2"), "label={label}");
        assert!(label.contains("22.5"), "label={label}");
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
        let (label, _) = sc.item_options(0).unwrap();
        assert!(label.contains("размер M"), "label={label}");
        assert!(label.contains("повреждён: да"), "label={label}");
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
}

