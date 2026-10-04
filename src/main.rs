use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame, layout::{Constraint, Direction, Layout}, style::{Color, Style}, widgets::{Block, Borders, List, ListState, Paragraph},
};


use ratatui_textarea::TextArea;

mod service_center;
use service_center::{ServiceCenter, VehicleTypes, ItemTypes};

use strum::Display;
use strum::IntoEnumIterator;

#[derive(Debug, Clone, Copy, PartialEq, Display, Default)]
pub enum PageOption {
    #[strum(to_string = "главное меню")]
    #[default]
    MainMenu,
    #[strum(to_string = "добавить новый транспорт")]
    AddNewTransport,
    #[strum(to_string = "добавить новый предмет")]
    AddNewItem,
    #[strum(to_string = "провести техосмотр")]
    TechReview,
    #[strum(to_string = "статистика потребления энергии")]
    EnergyStatistic,
    #[strum(to_string = "список устройств для новичков")]
    NubList,
    #[strum(to_string = "инвентарь")]
    Inventory,
    #[strum(to_string = "о нас")]
    AboutUs,
    #[strum(to_string = "добавление транспорта")]
    AddNewTransportForm(VehicleTypes),
    #[strum(to_string = "добавление предмета")]
    AddNewItemForm(ItemTypes),
    #[strum(to_string = "сохранить")]
    SaveToFile,
    #[strum(to_string = "загрузить")]
    LoadFromFile,
    #[strum(to_string = "Произошла ошибка!")]
    ErrorPage
}

const ABOUT_US_TEXT: &str = include_str!("about_us.txt");

#[derive(Debug, Clone, Copy)]
enum Action {
    None,
    Back,
    Quit,
    Go(PageOption),
    Save,
    Load
}

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::new().run(terminal))
}

trait IPage {
    fn draw(&mut self, frame: &mut Frame, sc: &mut ServiceCenter);
    fn handle_key_event(&mut self, key_event: KeyEvent, sc: &mut ServiceCenter);
    fn back(&self) -> PageOption;
    fn move_to(&mut self, sc: &mut ServiceCenter) -> Action;
}

struct ListPage {
    options: Vec<(String, Action)>,
    cur: usize,
    title: String,
    backpage: PageOption,
    moving: Action,
}

impl ListPage {
    fn new(title: impl Into<String>, backpage: PageOption, options: Vec<(String, Action)>) -> Self {
        Self {
            options,
            cur: 0,
            title: title.into(),
            backpage,
            moving: Action::None,
        }
    }
}

impl IPage for ListPage {
    fn draw(&mut self, frame: &mut Frame, _sc: &mut ServiceCenter) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(vec![
                Constraint::Length(0),
                Constraint::Fill(1),
                Constraint::Length(1)
            ])
            .split(frame.area());

        let title_text = Paragraph::new(self.title.to_string())
                .style(Style::default().bg(Color::Indexed(236)));
        frame.render_widget(title_text, chunks[0]);
        
        let items: Vec<String> = self.options.iter().map(|(label, _)| label.clone()).collect();

        let list = List::new(items)
            .block(Block::default().title(self.title.as_str()).borders(Borders::ALL))
            .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black))
            .highlight_symbol(">> ");

        let mut state = ListState::default();
        state.select(Some(self.cur));
        frame.render_stateful_widget(list, chunks[1], &mut state);

        let subtext = Paragraph::new("Esc/Left - назад; up/down; Enter/Right - ок".to_string())
                .style(Style::default().bg(Color::Indexed(236)));
        frame.render_widget(subtext, chunks[2]);
    }

    fn handle_key_event(&mut self, key_event: KeyEvent, _sc: &mut ServiceCenter) {
        if key_event.kind != KeyEventKind::Press {
            return;
        }

        let last = self.options.len().saturating_sub(1);
        match key_event.code {
            KeyCode::Up => self.cur = self.cur.saturating_sub(1),
            KeyCode::Down => self.cur = (self.cur + 1).min(last),
            KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Right => {
                if let Some((_, action)) = self.options.get(self.cur) {
                    self.moving = *action;
                }
            }
            _ => {}
        }
    }

    fn back(&self) -> PageOption {
        self.backpage
    }

    fn move_to(&mut self, _sc: &mut ServiceCenter) -> Action {
        std::mem::replace(&mut self.moving, Action::None)
    }
}

type AreaVec = Vec<(String, TextArea<'static>)>;
pub type BuildAreasFnT = fn (&mut AreaVec) -> ();
pub type DoneFnT = fn (&AreaVec, &mut ServiceCenter) -> ();

struct FormPage {
    title: String,
    backpage: PageOption,
    areas: AreaVec,
    focus: usize,
    done_fn: DoneFnT,
    moving: Action
}

impl FormPage {
    fn new(title: String, backpage: PageOption, build_areas_fn: BuildAreasFnT,
        done_fn: DoneFnT) -> Self {
        let mut ret = FormPage {
            title, backpage, areas: vec![], focus: 0, done_fn, moving: Action::None
        };
        build_areas_fn(&mut ret.areas);
        ret
    }
}

impl IPage for FormPage {
    fn draw(&mut self, frame: &mut Frame, _sc: &mut ServiceCenter) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                vec![Constraint::Length(1)],
                vec![Constraint::Length(3); self.areas.len()],
                vec![Constraint::Length(1)],
            ].concat())
            .split(frame.area());

        let title_text = Paragraph::new(self.title.to_string())
                .style(Style::default().bg(Color::Indexed(236)));
        frame.render_widget(title_text, chunks[0]);

        let focus = self.focus;
        let get_style = |ind| {
            if focus == ind {
                Style::default().fg(Color::Green).bold()
            } else {
                Style::default().fg(Color::DarkGray)
            }
        };

        for (ind, (title, area)) in self.areas.iter_mut().enumerate() {
            area.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title.clone())
                    .border_style(get_style(ind)),
            );
            frame.render_widget(&*area, chunks[ind+1]);
        }

        let subtext = Paragraph::new("Esc/Left - отмена; Enter/Tab - далее".to_string())
                .style(Style::default().bg(Color::Indexed(236)));
        frame.render_widget(subtext, chunks[self.areas.len()+1]);
    }
    fn handle_key_event(&mut self, key_event: KeyEvent, sc: &mut ServiceCenter) {
        if key_event.kind == KeyEventKind::Press {
            match key_event.code {
                KeyCode::Tab | KeyCode::Enter => {
                    if self.focus + 1 == self.areas.len() {
                        (self.done_fn)(&self.areas, sc);
                        self.moving = Action::Back
                    } else {
                        self.focus += 1
                    }
                },
                _ => {
                    if let Some((_, area)) = self.areas.get_mut(self.focus) {
                        area.input(key_event);
                    }
                },
            }
        }
    }
    fn back(&self) -> PageOption {
        self.backpage
    }
    fn move_to(&mut self, _sc: &mut ServiceCenter) -> Action {
        self.moving
    }
}

struct TextPage {
    text: String,
    backpage: PageOption,
}

impl IPage for TextPage {
    fn draw(&mut self, frame: &mut Frame, _sc: &mut ServiceCenter) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Length(1)
            ])
            .split(frame.area());

        let paragraph = Paragraph::new(self.text.clone())
            .block(Block::default().title("Текст").borders(Borders::ALL));
        frame.render_widget(paragraph, chunks[0]);

        let subtext = Paragraph::new("Esc/Left - назад".to_string())
                .style(Style::default().bg(Color::Indexed(236)));
        frame.render_widget(subtext, chunks[1]);
    }
    fn handle_key_event(&mut self, _key_event: KeyEvent, _sc: &mut ServiceCenter) {}
    fn back(&self) -> PageOption {
        self.backpage
    }
    fn move_to(&mut self, _sc: &mut ServiceCenter) -> Action {
        Action::None
    }
}

enum Page {
    List(ListPage),
    Form(FormPage),
    Text(TextPage),
}

impl IPage for Page {
    fn draw(&mut self, frame: &mut Frame, sc: &mut ServiceCenter) {
        match self {
            Page::List(p) => p.draw(frame, sc),
            Page::Form(p) => p.draw(frame, sc),
            Page::Text(p) => p.draw(frame, sc),
        }
    }
    fn handle_key_event(&mut self, key_event: KeyEvent, sc: &mut ServiceCenter) {
        match self {
            Page::List(p) => p.handle_key_event(key_event, sc),
            Page::Form(p) => p.handle_key_event(key_event, sc),
            Page::Text(p) => p.handle_key_event(key_event, sc),
        }
    }
    fn back(&self) -> PageOption {
        match self {
            Page::List(p) => p.back(),
            Page::Form(p) => p.back(),
            Page::Text(p) => p.back(),
        }
    }
    fn move_to(&mut self, sc: &mut ServiceCenter) -> Action {
        match self {
            Page::List(p) => p.move_to(sc),
            Page::Form(p) => p.move_to(sc),
            Page::Text(p) => p.move_to(sc),
        }
    }
}

struct App {
    sc: ServiceCenter,
    page: Page
}

impl App {
    fn new() -> Self {
        let mut sc = ServiceCenter::new();
        sc.load();
        let page = Self::build_page(PageOption::MainMenu, &sc);
        Self { sc, page }
    }

    fn menu_item(page: PageOption) -> (String, Action) {
        (page.to_string(), Action::Go(page))
    }

    fn main_menu_options() -> Vec<(String, Action)> {
        vec![
            Self::menu_item(PageOption::AddNewTransport),
            Self::menu_item(PageOption::AddNewItem),
            Self::menu_item(PageOption::TechReview),
            Self::menu_item(PageOption::EnergyStatistic),
            Self::menu_item(PageOption::NubList),
            Self::menu_item(PageOption::Inventory),
            Self::menu_item(PageOption::AboutUs),
            Self::menu_item(PageOption::SaveToFile),
            Self::menu_item(PageOption::LoadFromFile),
            ("выход (exit)".to_string(), Action::Quit),
        ]
    }

    fn list_options(items: Vec<String>) -> Vec<(String, Action)> {
        items.into_iter().map(|s| (s, Action::None)).collect()
    }

    fn vehicle_form(vt: VehicleTypes) -> Page {
        let (title, build_areas_fn, done_fn) = vt.unpack();
        Page::Form(FormPage::new(
            title,
            PageOption::AddNewTransport,
            build_areas_fn,
            done_fn,
        ))
    }

    fn item_form(it: ItemTypes) -> Page {
        let (title, build_areas_fn, done_fn) = it.unpack();
        Page::Form(FormPage::new(
            title,
            PageOption::AddNewItem,
            build_areas_fn,
            done_fn,
        ))
    }

    fn build_page(page: PageOption, sc: &ServiceCenter) -> Page {
        match page {
            PageOption::MainMenu => Page::List(ListPage::new(
                "Главное меню",
                PageOption::MainMenu,
                Self::main_menu_options(),
            )),
            PageOption::AboutUs => Page::Text(TextPage {
                text: ABOUT_US_TEXT.to_string(),
                backpage: PageOption::MainMenu,
            }),
            PageOption::TechReview => Page::List(ListPage::new(
                "Техосмотр",
                PageOption::MainMenu,
                Self::list_options(sc.transport_list()),
            )),
            PageOption::Inventory => Page::List(ListPage::new(
                "Инвентарь",
                PageOption::MainMenu,
                Self::list_options(sc.item_list()),
            )),
            PageOption::AddNewTransport => Page::List(ListPage::new(
                "Создание транспорта (выберете что создать)",
                PageOption::MainMenu,
                VehicleTypes::iter()
                        .map(|t| (t.to_string(), Action::Go(PageOption::AddNewTransportForm(t))))
                        .collect()
            )),
            PageOption::AddNewTransportForm(vt) => Self::vehicle_form(vt),
            PageOption::AddNewItem => Page::List(ListPage::new(
                "Создание предмета (выберете что создать)",
                PageOption::MainMenu,
                ItemTypes::iter()
                        .map(|t| (t.to_string(), Action::Go(PageOption::AddNewItemForm(t))))
                        .collect()
            )),
            PageOption::AddNewItemForm(it) => Self::item_form(it),
            PageOption::EnergyStatistic => {
                Page::List(ListPage::new(
                    "Затраты энергии",
                    PageOption::MainMenu,
                    sc.energy_statistic().iter()
                    .map(|s| (s.clone(), Action::None))
                    .collect()
                ))
            },
            PageOption::NubList => todo!(),
            PageOption::SaveToFile => {
                Page::List(ListPage::new(
                    "Сохраненине в файл. Вы уверены?",
                    PageOption::MainMenu,
                    vec![("Подтвердить".to_string(), Action::Save), ("Отменить".to_string(), Action::Back)]
                ))
            },
            PageOption::LoadFromFile => {
                Page::List(ListPage::new(
                    "Загрузка из файла. Вы уверены?",
                    PageOption::MainMenu,
                    vec![("Подтвердить".to_string(), Action::Load), ("Отменить".to_string(), Action::Back)]
                ))
            },
            PageOption::ErrorPage => {
                Page::Text(TextPage {
                    text: "ошибка: ошибок пока не видно".to_string(),
                    backpage: PageOption::MainMenu
                })
            }
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
            match self.page.move_to(&mut self.sc) {
                Action::Back => self.page = Self::build_page(self.page.back(), &self.sc),
                Action::None => {}
                Action::Quit => { break; },
                Action::Go(p) => self.page = Self::build_page(p, &self.sc),
                Action::Load => match self.sc.load() {
                    Err(e) => {
                        let error = format!("Err: {}", e);
                        self.page = Page::Text(TextPage {
                            text: format!("Упс! Произошла ошибка: {}", error),
                            backpage: PageOption::MainMenu
                        });
                    },
                    Ok(()) => self.page = Self::build_page(PageOption::MainMenu, &self.sc)
                },
                Action::Save => match self.sc.save() {
                    Err(e) => {
                        let error = format!("Err: {}", e);
                        self.page = Page::Text(TextPage {
                            text: format!("Упс! Произошла ошибка: {}", error),
                            backpage: PageOption::MainMenu
                        });
                    },
                    Ok(()) => self.page = Self::build_page(PageOption::MainMenu, &self.sc)
                }
            }
        }
        self.sc.save();
        return Ok(());
    }

    fn draw(&mut self, frame: &mut Frame) {
        self.page.draw(frame, &mut self.sc)
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if let Event::Key(key_event) = event::read()? {
            if key_event.kind == KeyEventKind::Press {
                if key_event.code == KeyCode::Esc || key_event.code == KeyCode::Left {
                    let back = self.page.back();
                    self.page = Self::build_page(back, &self.sc);
                } else {
                    self.page.handle_key_event(key_event, &mut self.sc);
                }
            }
        }
        Ok(())
    }
}
