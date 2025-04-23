mod logging;
use logging::Level;

use iced::widget::{Column, Row, Scrollable, scrollable};
use iced::widget::{Container, container, row, text}; // Import Container
use iced::{
    Alignment, Application, Color, Command, Element, Font, Length, Renderer, Settings, Size, Theme,
    alignment, executor, font, window,
};
use serde::Deserialize;
use std::borrow::Cow;

/**
Emoji data structure
*/
#[derive(Debug, Clone, Deserialize)]
struct EmojiData {
    emoji: String,
    keywords: String,
    category: String,
}

/**
Application state struct
*/
struct NicePickApp {
    emojis: Vec<EmojiData>,  // Field to store emoji data
    emoji_font_loaded: bool, // Flag to track if the emoji font is loaded
}

/**
Define the messages the application can react to (none for now)
*/
#[derive(Debug, Clone)]
enum Message {
    FontLoaded(Result<(), font::Error>), // Message to signal font loading result
}

/**
Load the font bytes for an emoji font, for now hardcoding to Noto Color Emoji
*/
const NOTO_COLOR_EMOJI_BYTES: &[u8] = include_bytes!("../assets/NotoColorEmoji-Regular.ttf");

/**
Constant for the emoji font
*/
const EMOJI_FONT: Font = Font::with_name("Noto Color Emoji");

/**
Implementation of the Application trait for our state
*/
impl Application for NicePickApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    /**
    Initialize the application state and load emoji data.
    @params _flags: Flags for the application
    @return (Self, Command<Message>) Initialize the application state and load emoji data.
    */
    fn new(_flags: ()) -> (Self, Command<Message>) {
        // If debug logging is enabled, record the JSON load time
        dbug!("Initializing NicePickApp state (requesting font load)...");
        let start_time = if logging::log_enabled(Level::Debug) {
            Some(std::time::Instant::now())
        } else {
            None
        };

        // Load and parse JSON emoji data
        let json_data = include_str!("../data.json");
        let emojis: Vec<EmojiData> =
            serde_json::from_str(json_data).expect("Failed to parse data.json");

        // Count final emoji JSON data load time (if debug logging is enabled)
        if let Some(start) = start_time {
            let duration = start.elapsed();
            dbug!("JSON emoji data loaded in {:?}", duration);
        }

        info!("JSON emoji data loaded successfully");

        // Loaded emojis get stored in app state
        (
            NicePickApp {
                emojis,
                emoji_font_loaded: false, // Font is not loaded initially
            },
            font::load(Cow::Borrowed(NOTO_COLOR_EMOJI_BYTES)).map(Message::FontLoaded),
        )
    }

    /**
    Application title function
    @param &self: Self reference
    @return String: Application title
    */
    fn title(&self) -> String {
        String::from("nICEpick")
    }

    /**
    Application update function
    @param &mut self: Mutable self reference
    @param _message: Message to update the application state
    @return Command<Message>: Command to execute after updating the application state
    */
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::FontLoaded(Ok(())) => {
                self.emoji_font_loaded = true;
                info!("Emoji font loaded successfully.");
                Command::none()
            }
            Message::FontLoaded(Err(e)) => {
                fail!("Failed to load emoji font: {:?}", e);
                // Keep emoji_font_loaded as false
                Command::none()
            }
        }
    }

    /**
    Application view function
    @param &self: Self reference
    @return Element<Message>: Element to display the application state
    */
    fn view(&self) -> Element<Message> {
        // Start timer for view function if debug logging is enabled
        let start_time = if logging::log_enabled(Level::Debug) {
            Some(std::time::Instant::now())
        } else {
            None
        };
        const ITEMS_PER_ROW: usize = 4;
        const SPACING: u16 = 10;

        // Create rows of emojis
        let mut rows = Vec::new();
        for chunk in self.emojis.chunks(ITEMS_PER_ROW) {
            let mut row_elements: Row<'_, Message, Theme, Renderer> = Row::new().spacing(SPACING);
            for item in chunk {
                // Add each emoji as text with the correct font
                let emoji_text = if self.emoji_font_loaded {
                    // Use the emoji font if loaded
                    text(&item.emoji).font(EMOJI_FONT).size(32)
                } else {
                    // Use a placeholder or default font if not loaded yet
                    text("⏳").size(32)
                };
                row_elements = row_elements.push(emoji_text);
            }
            rows.push(row_elements);
        }

        // Create a column containing all the rows
        let content = Column::with_children(rows.into_iter().map(Element::from))
            .spacing(SPACING)
            .padding(SPACING); // Add padding around the grid

        // Wrap the content in a scrollable container
        let scrollable_content = scrollable(content).width(Length::Fill).height(Length::Fill);

        // Wrap the scrollable in a container for background and centering
        let final_element = container(scrollable_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(|_theme: &Theme| container::Appearance {
                background: Some(Color::from_rgb8(40, 44, 52).into()),
                ..container::Appearance::default()
            })
            .into();

        // Log duration if debug logging is enabled
        if let Some(start) = start_time {
            let duration = start.elapsed();
            dbug!("View construction took {:?}", duration);
        }

        final_element
    }

    fn theme(&self) -> Theme {
        Theme::default()
    }
}

/**
Main entrypoint of the application
@returns Iced application
*/
fn main() -> iced::Result {
    let main_start_time = std::time::Instant::now();

    // Initialize logging
    logging::init(Level::Debug);

    dbug!("Logger initialized in {:?}", main_start_time.elapsed());

    info!("Configuring application settings");

    let settings = Settings {
        window: window::Settings {
            size: Size::new(400.0, 200.0),
            decorations: false,
            transparent: true,
            ..window::Settings::default()
        },
        // Let Iced use its default text font
        ..Settings::default()
    };

    let setup_duration = main_start_time.elapsed();
    dbug!("Application setup (before run) took {:?}", setup_duration);
    info!("Starting Iced event loop (NicePickApp::run)...");

    NicePickApp::run(settings)
}
