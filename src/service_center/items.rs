use std::rc::Rc;
use ratatui_textarea::TextArea;
use serde::{Serialize, Deserialize};

use super::{AreaVec, ServiceCenter, Wh};
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
    fn done(areas: &AreaVec, sc: &mut ServiceCenter) where Self: Sized; // добавляем предмет
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
    fn parse(s: &str) -> Self {
        match s.trim().to_uppercase().as_str() {
            "M" => ClothingSize::M,
            "L" => ClothingSize::L,
            "XL" => ClothingSize::XL,
            _ => ClothingSize::S,
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

    fn done(areas: &AreaVec, sc: &mut ServiceCenter) {
        let port_count = areas[0].1.lines().join("").trim().parse().unwrap_or(0);
        let charge_speed = areas[1].1.lines().join("").trim().parse().unwrap_or(0.0);
        let id = sc.next_item_id();
        sc.add_item(Rc::new(ChargingStation {
            id,
            port_count,
            charge_speed: Wh(charge_speed),
        }));
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

    fn done(areas: &AreaVec, sc: &mut ServiceCenter) {
        let size = ClothingSize::parse(&areas[0].1.lines().join(""));
        let text = areas[1].1.lines().join("");
        let is_damaged = matches!(
            text.trim().to_lowercase().as_str(),
            "y" | "yes" | "1" | "true" | "да"
        );
        let id = sc.next_item_id();
        sc.add_item(Rc::new(Helmet { id, size, is_damaged }));
    }
}

