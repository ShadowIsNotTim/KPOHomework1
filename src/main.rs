use std::io;
use std::rc::Rc;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame, layout::{Constraint, Direction, Layout}, style::{Color, Modifier, Style}, widgets::{Block, Borders, List, ListState, Paragraph},
};


use ratatui_textarea::TextArea;

mod service_center;
use service_center::{ServiceCenter, VehicleTypes, ItemTypes, FormatError, Vehicle, Item};

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
    #[strum(to_string = "список техники")]
    TransportList,
    #[strum(to_string = "статистика потребления энергии")]
    EnergyStatistics,
    #[strum(to_string = "список устройств для новичков")]
    BeginnersList,
    #[strum(to_string = "инвентарь")]
    Inventory,
    #[strum(to_string = "о нас")]
    AboutUs,
    #[strum(to_string = "добавление транспорта")]
    VehicleForm(VehicleTypes),
    #[strum(to_string = "добавление предмета")]
    ItemForm(ItemTypes),
    #[strum(to_string = "редактирование транспорта")]
    VehicleEdit(usize),
    #[strum(to_string = "редактирование предмета")]
    ItemEdit(usize),
    #[strum(to_string = "операции с данными")]
    DataOperations,
    #[strum(to_string = "техосмотр")]
    TechReview,
    #[strum(to_string = "выход (exit)")]
    Exit,
    #[strum(to_string = "Произошла ошибка!")]
    Error
}

const ABOUT_US_TEXT: &str = include_str!("about_us.txt");

#[derive(Debug, Clone, Copy)]
pub enum Action {
    None,
    Back,
    Go(PageOption),
    Save,
    Load,
    LoadExample,
    VehicleInfo(usize),
    ItemInfo(usize)
}

#[derive(Debug)]
enum ExitError {
    Exit,
    Io(io::Error),
}

impl From<io::Error> for ExitError {
    fn from(error: io::Error) -> Self {
        ExitError::Io(error)
    }
}

impl std::fmt::Display for ExitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExitError::Exit => write!(f, "выход из приложения"),
            ExitError::Io(error) => write!(f, "{}", error),
        }
    }
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

type OptionsFnT = fn(&ServiceCenter, usize) -> Option<(String, Action)>;

struct ListPage {
    get_option: OptionsFnT,
    cur: usize,
    title: String,
    backpage: PageOption,
    moving: Action,
    subtext2: String,
    custom_keys: Vec<(char, fn(&mut ServiceCenter, usize) -> Action)>,
}

impl ListPage {
    fn new(title: impl Into<String>, backpage: PageOption, get_option: OptionsFnT) -> Self {
        Self {
            get_option,
            cur: 0,
            title: title.into(),
            backpage,
            moving: Action::None,
            subtext2: String::new(),
            custom_keys: Vec::new(),
        }
    }

    fn set_key_code_event(mut self, key: char, description: String,
        action: fn(&mut ServiceCenter, usize) -> Action) -> Self {
        if !self.subtext2.is_empty() {
            self.subtext2.push_str("; ");
        }
        self.subtext2.push_str(&format!("{} - {}", key, description));
        self.custom_keys.push((key, action));
        self
    }

    fn options_len(&self, sc: &ServiceCenter) -> usize {
        let get_option = self.get_option;
        let mut len = 0;
        while get_option(sc, len).is_some() {
            len += 1;
        }
        len
    }
}

impl IPage for ListPage {
    fn draw(&mut self, frame: &mut Frame, sc: &mut ServiceCenter) {
        let len = self.options_len(sc);
        if self.cur >= len {
            self.cur = len.saturating_sub(1);
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(vec![
                Constraint::Length(0),
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Length(1)
            ])
            .split(frame.area());

        let title_text = Paragraph::new(self.title.to_string())
                .style(Style::default().bg(Color::Indexed(236)));
        frame.render_widget(title_text, chunks[0]);
        
        let get_option = self.get_option;
        let items: Vec<String> = (0..len)
            .filter_map(|ind| get_option(sc, ind).map(|(label, _)| label))
            .collect();

        let list = List::new(items)
            .block(Block::default().title(self.title.as_str()).borders(Borders::ALL))
            .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black))
            .highlight_symbol(">> ");

        let mut state = ListState::default();
        state.select(Some(self.cur));
        frame.render_stateful_widget(list, chunks[1], &mut state);

        let custom_text = Paragraph::new(self.subtext2.clone())
                .style(Style::default().bg(Color::Indexed(236)));
        frame.render_widget(custom_text, chunks[2]);

        let subtext = Paragraph::new("Esc/Left - назад; up/down; Enter/Right - ок".to_string())
                .style(Style::default().bg(Color::Indexed(236)));
        frame.render_widget(subtext, chunks[3]);
    }

    fn handle_key_event(&mut self, key_event: KeyEvent, sc: &mut ServiceCenter) {
        if key_event.kind != KeyEventKind::Press {
            return;
        }

        let last = self.options_len(sc).saturating_sub(1);
        match key_event.code {
            KeyCode::Up => self.cur = self.cur.saturating_sub(1),
            KeyCode::Down => self.cur = (self.cur + 1).min(last),
            KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Right => {
                if let Some((_, action)) = (self.get_option)(sc, self.cur) {
                    self.moving = action;
                }
            },
            KeyCode::Left => {
                self.moving = Action::Back;
            }
            KeyCode::Char(c) => {
                let action = self.custom_keys.iter()
                    .find(|(key, _)| *key == c)
                    .map(|(_, action)| *action);
                if let Some(action) = action {
                    self.moving = action(sc, self.cur);
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
pub type DoneFnT = fn (&AreaVec, &mut ServiceCenter) -> Result<(), FormatError>;

type BuildAreasFn = Box<dyn FnMut(&mut AreaVec)>;
type DoneFn = Box<dyn FnMut(&AreaVec, &mut ServiceCenter) -> Result<(), FormatError>>;

struct FormPage {
    title: String,
    backpage: PageOption,
    areas: AreaVec,
    focus: usize,
    done_fn: DoneFn,
    format_error: Result<(), FormatError>,
    moving: Action
}

impl FormPage {
    fn from_parts(title: String, backpage: PageOption, mut build_areas: BuildAreasFn,
        done_fn: DoneFn) -> Self {
        let mut ret = FormPage {
            title, backpage, areas: vec![], focus: 0, done_fn,
            format_error: Ok(()), moving: Action::None
        };
        build_areas(&mut ret.areas);
        ret
    }

    fn new(title: String, backpage: PageOption, build_areas_fn: BuildAreasFnT,
        done_fn: DoneFnT) -> Self {
        Self::from_parts(
            title,
            backpage,
            Box::new(move |areas| build_areas_fn(areas)),
            Box::new(move |areas, sc| done_fn(areas, sc)),
        )
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
            if ind == focus {
                area.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
            } else {
                area.set_cursor_style(Style::default());
            }
            frame.render_widget(&*area, chunks[ind+1]);
        }

        let error_text = match &self.format_error {
            Ok(()) => String::new(),
            Err(error) => format!("Ошибка: {}", error),
        };
        let error_line = Paragraph::new(error_text)
                .style(Style::default().fg(Color::Red));
        frame.render_widget(error_line, chunks[self.areas.len()+1]);

        let subtext = Paragraph::new("Esc - отмена; Up/Down - по полям; Enter/Tab - далее; Left/Right - по тексту".to_string())
                .style(Style::default().bg(Color::Indexed(236)));
        frame.render_widget(subtext, chunks[self.areas.len()+2]);
    }
    fn handle_key_event(&mut self, key_event: KeyEvent, sc: &mut ServiceCenter) {
        if key_event.kind == KeyEventKind::Press {
            match key_event.code {
                KeyCode::Up => {
                    self.focus = self.focus.saturating_sub(1);
                },
                KeyCode::Down | KeyCode::Tab | KeyCode::Enter => {
                    if self.focus + 1 == self.areas.len() {
                        self.format_error = (self.done_fn)(&self.areas, sc);
                        if self.format_error.is_ok() {
                            self.moving = Action::Back
                        }
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

        let subtext = Paragraph::new("Esc - назад".to_string())
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
    page: Page,
    history: Vec<Page>
}

impl App {
    fn new() -> Self {
        let mut sc = ServiceCenter::new();
        let page = match sc.load() {
            Ok(()) => Self::build_page(PageOption::MainMenu, &sc)
                .unwrap_or_else(|e| Self::error_page(e)),
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => Self::build_page(PageOption::AboutUs, &sc)
                    .unwrap_or_else(|e| Self::error_page(e)),
                _ => Page::Text(TextPage { text: format!("ошибка при загрузке date.json: {}", e), backpage: PageOption::MainMenu })
            }
        };
        Self { sc, page, history: Vec::new() }
    }

    fn menu_item(page: PageOption) -> (String, Action) {
        (page.to_string(), Action::Go(page))
    }

    fn main_menu_options() -> Vec<(String, Action)> {
        vec![
            Self::menu_item(PageOption::AddNewTransport),
            Self::menu_item(PageOption::AddNewItem),
            Self::menu_item(PageOption::TransportList),
            Self::menu_item(PageOption::EnergyStatistics),
            Self::menu_item(PageOption::BeginnersList),
            Self::menu_item(PageOption::Inventory),
            Self::menu_item(PageOption::AboutUs),
            Self::menu_item(PageOption::TechReview),
            Self::menu_item(PageOption::DataOperations),
            Self::menu_item(PageOption::Exit),
        ]
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

    fn vehicle_edit_form(v: Rc<dyn Vehicle>, ind: usize) -> Page {
        let title = format!("Редактирование: {}", v.name());
        let build = {
            let v = v.clone();
            Box::new(move |areas: &mut AreaVec| v.build_edit_areas(areas))
        };
        let done = Box::new(move |areas: &AreaVec, sc: &mut ServiceCenter| {
            let updated = v.done_edit(areas)?;
            sc.replace_transport(ind, updated);
            Ok(())
        });
        Page::Form(FormPage::from_parts(
            title, PageOption::TransportList, build, done))
    }

    fn item_edit_form(i: Rc<dyn Item>, ind: usize) -> Page {
        let title = format!("Редактирование: {}", i.name());
        let build = {
            let i = i.clone();
            Box::new(move |areas: &mut AreaVec| i.build_edit_areas(areas))
        };
        let done = Box::new(move |areas: &AreaVec, sc: &mut ServiceCenter| {
            let updated = i.done_edit(areas)?;
            sc.replace_item(ind, updated);
            Ok(())
        });
        Page::Form(FormPage::from_parts(
            title, PageOption::Inventory, build, done))
    }

    fn not_editable_page(name: &str, backpage: PageOption) -> Page {
        Page::Text(TextPage {
            text: format!("«{}» нельзя редактировать >:-)\nУдали его!", name),
            backpage,
        })
    }

    fn build_page(page: PageOption, sc: &ServiceCenter) -> Result<Page, ExitError> {
        match page {
            PageOption::MainMenu => Ok(Page::List(ListPage::new(
                "Главное меню",
                PageOption::Exit,
                main_menu_option,
            ))),
            PageOption::AboutUs => Ok(Page::Text(TextPage {
                text: ABOUT_US_TEXT.to_string(),
                backpage: PageOption::MainMenu,
            })),
            PageOption::TransportList => Ok(Page::List(
                ListPage::new(
                    "Список техники",
                    PageOption::MainMenu,
                    ServiceCenter::transport_options,
                ).set_key_code_event('d', "удалить".to_string(),
                    |sc, ind| { sc.delete_transport(ind); Action::None })
                 .set_key_code_event('e', "редактировать".to_string(),
                    |_sc, ind| Action::Go(PageOption::VehicleEdit(ind)))
                 .set_key_code_event('t', "взять".to_string(),
                    |sc, ind| { sc.set_vehicle_taken(ind, true); Action::None })
                 .set_key_code_event('p', "положить".to_string(),
                    |sc, ind| { sc.set_vehicle_taken(ind, false); Action::None })
            )),
            PageOption::Inventory => Ok(Page::List(
                ListPage::new(
                    "Инвентарь",
                    PageOption::MainMenu,
                    ServiceCenter::item_options,
                ).set_key_code_event('d', "удалить".to_string(),
                    |sc, ind| { sc.delete_item(ind); Action::None })
                 .set_key_code_event('e', "редактировать".to_string(),
                    |_sc, ind| Action::Go(PageOption::ItemEdit(ind)))
                 .set_key_code_event('t', "взять".to_string(),
                    |sc, ind| { sc.set_item_taken(ind, true); Action::None })
                 .set_key_code_event('p', "положить".to_string(),
                    |sc, ind| { sc.set_item_taken(ind, false); Action::None })
            )),
            PageOption::AddNewTransport => Ok(Page::List(ListPage::new(
                "Создание транспорта (выберете что создать)",
                PageOption::MainMenu,
                vehicle_type_option,
            ))),
            PageOption::VehicleForm(vt) => Ok(Self::vehicle_form(vt)),
            PageOption::AddNewItem => Ok(Page::List(ListPage::new(
                "Создание предмета (выберете что создать)",
                PageOption::MainMenu,
                item_type_option,
            ))),
            PageOption::ItemForm(it) => Ok(Self::item_form(it)),
            PageOption::EnergyStatistics => {
                Ok(Page::List(ListPage::new(
                    "Затраты энергии",
                    PageOption::MainMenu,
                    ServiceCenter::energy_statistic_options,
                )))
            },
            PageOption::BeginnersList => {
                Ok(Page::List(ListPage::new(
                    "Список техники для новичков 🥴",
                    // раст работает на UTF-8 поэтому поддерживает смайлики нативно 🥳🦄
                    PageOption::MainMenu,
                    ServiceCenter::nublist_options,
                ).set_key_code_event('t', "взять".to_string(),
                    |sc, ind| { sc.set_beginner_taken(ind, true); Action::None })
                ))
            },
            PageOption::DataOperations => {
                Ok(Page::List(ListPage::new(
                    "Операции с данными",
                    PageOption::MainMenu,
                    data_operation_option,
                )))
            },
            PageOption::TechReview => {
                Ok(Page::List(
                    ListPage::new(
                        "Техосмотр",
                        PageOption::MainMenu,
                        ServiceCenter::tech_review_options,
                    ).set_key_code_event('d', "удалить".to_string(),
                        |sc, ind| { sc.delete_reviewed(ind); Action::None })
                     .set_key_code_event('e', "редактировать".to_string(),
                        review_edit_action)
                     .set_key_code_event('t', "взять".to_string(),
                        |sc, ind| { sc.set_reviewed_taken(ind, true); Action::None })
                     .set_key_code_event('p', "положить".to_string(),
                        |sc, ind| { sc.set_reviewed_taken(ind, false); Action::None })
                ))
            },
            PageOption::VehicleEdit(ind) => match sc.transport_at(ind) {
                None => Ok(Self::error_page("нет такого транспорта")),
                Some(v) if !v.is_editable() =>
                    Ok(Self::not_editable_page(&v.name(), PageOption::TransportList)),
                Some(v) => Ok(Self::vehicle_edit_form(v, ind)),
            },
            PageOption::ItemEdit(ind) => match sc.item_at(ind) {
                None => Ok(Self::error_page("нет такого предмета")),
                Some(i) if !i.is_editable() =>
                    Ok(Self::not_editable_page(&i.name(), PageOption::Inventory)),
                Some(i) => Ok(Self::item_edit_form(i, ind)),
            },
            PageOption::Exit => Err(ExitError::Exit),
            PageOption::Error => {
                Ok(Page::Text(TextPage {
                    text: "ошибка: ошибок пока не видно".to_string(),
                    backpage: PageOption::MainMenu
                }))
            }
        }
    }

    fn error_page(error: impl std::fmt::Display) -> Page {
        Page::Text(TextPage {
            text: format!("Упс! Произошла ошибка: {}", error),
            backpage: PageOption::MainMenu,
        })
    }

    fn info_page(text: Option<String>, backpage: PageOption) -> Page {
        Page::Text(TextPage {
            text: text.unwrap_or_else(|| "нет данных".to_string()),
            backpage,
        })
    }

    fn navigate(&mut self, page: Page) {
        self.history.push(std::mem::replace(&mut self.page, page));
    }

    fn go_back(&mut self) -> Result<(), ExitError> {
        match self.history.pop() {
            Some(previous) => {
                self.page = previous;
                Ok(())
            }
            None => {
                self.page = Self::build_page(self.page.back(), &self.sc)?;
                Ok(())
            }
        }
    }

    fn go_home(&mut self) -> Result<(), ExitError> {
        if self.history.is_empty() {
            self.page = Self::build_page(PageOption::MainMenu, &self.sc)?;
        } else {
            let main = self.history.remove(0);
            self.history.clear();
            self.page = main;
        }
        Ok(())
    }

    fn go_to(&mut self, page: PageOption) -> Result<(), ExitError> {
        let page = Self::build_page(page, &self.sc)?;
        self.navigate(page);
        Ok(())
    }

    fn save_then_home(&mut self) -> Result<(), ExitError> {
        self.sc.save()?;
        self.go_home()
    }

    fn load_then_home(&mut self) -> Result<(), ExitError> {
        self.sc.load()?;
        self.go_home()
    }

    fn load_example_then_home(&mut self) -> Result<(), ExitError> {
        self.sc.load_example()?;
        self.go_home()
    }

    fn handle_nav(&mut self, result: Result<(), ExitError>) -> bool {
        match result {
            Ok(()) => false,
            Err(ExitError::Exit) => true,
            Err(error) => {
                self.navigate(Self::error_page(error));
                false
            }
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            let action = match self.handle_events() {
                Ok(action) => action,
                Err(ExitError::Exit) => break,
                Err(error) => {
                    self.navigate(Self::error_page(error));
                    continue;
                }
            };
            match action {
                Action::None => {}
                Action::Back => {
                    let result = self.go_back();
                    if self.handle_nav(result) { break; }
                }
                Action::Go(p) => {
                    let result = self.go_to(p);
                    if self.handle_nav(result) { break; }
                }
                Action::Save => {
                    let result = self.save_then_home();
                    if self.handle_nav(result) { break; }
                }
                Action::Load => {
                    let result = self.load_then_home();
                    if self.handle_nav(result) { break; }
                }
                Action::LoadExample => {
                    let result = self.load_example_then_home();
                    if self.handle_nav(result) { break; }
                }
                Action::VehicleInfo(ind) => {
                    let page = Self::info_page(
                        self.sc.transport_info(ind), PageOption::TransportList);
                    self.navigate(page);
                }
                Action::ItemInfo(ind) => {
                    let page = Self::info_page(
                        self.sc.item_info(ind), PageOption::Inventory);
                    self.navigate(page);
                }
            }
        }
        self.sc.save().ok();
        return Ok(());
    }

    fn draw(&mut self, frame: &mut Frame) {
        self.page.draw(frame, &mut self.sc)
    }

    fn handle_events(&mut self) -> Result<Action, ExitError> {
        if let Event::Key(key_event) = event::read()? {
            if key_event.kind == KeyEventKind::Press {
                if key_event.code == KeyCode::Esc {
                    self.go_back()?;
                } else {
                    self.page.handle_key_event(key_event, &mut self.sc);
                }
            }
        }
        Ok(self.page.move_to(&mut self.sc))
    }
}

fn main_menu_option(_sc: &ServiceCenter, ind: usize) -> Option<(String, Action)> {
    App::main_menu_options().into_iter().nth(ind)
}

fn data_operation_option(_sc: &ServiceCenter, ind: usize) -> Option<(String, Action)> {
    vec![
        ("сохранить".to_string(), Action::Save),
        ("загрузить".to_string(), Action::Load),
        ("восстановить до example.json".to_string(), Action::LoadExample),
    ].into_iter().nth(ind)
}

fn vehicle_type_option(_sc: &ServiceCenter, ind: usize) -> Option<(String, Action)> {
    VehicleTypes::iter()
        .nth(ind)
        .map(|t| (t.to_string(), Action::Go(PageOption::VehicleForm(t))))
}

fn item_type_option(_sc: &ServiceCenter, ind: usize) -> Option<(String, Action)> {
    ItemTypes::iter()
        .nth(ind)
        .map(|t| (t.to_string(), Action::Go(PageOption::ItemForm(t))))
}

fn review_edit_action(sc: &mut ServiceCenter, ind: usize) -> Action {
    match sc.tech_review_options(ind) {
        Some((_, Action::VehicleInfo(v))) => Action::Go(PageOption::VehicleEdit(v)),
        Some((_, Action::ItemInfo(i))) => Action::Go(PageOption::ItemEdit(i)),
        _ => Action::None,
    }
}

