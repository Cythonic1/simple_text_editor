use std::{
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

use iced::{
    executor,
    widget::{button, column, container, horizontal_space, row, text, text_editor},
    Application, Command, Element, Length, Settings, Theme,
};
#[derive(Default)]
struct Editor {
    path: Option<PathBuf>,
    content: text_editor::Content,
    error: Option<Error>,
}

fn main() -> iced::Result {
    Editor::run(Settings::default())
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    NewFile,
    FileOpened(Result<(PathBuf, Arc<String>), Error>),
    OpenFile,
}

// Command simple is allowing us to work Async task by returning some command or message to info of
// finishing and operation.
impl Application for Editor {
    type Message = Message;
    type Theme = Theme;
    type Flags = ();
    type Executor = executor::Default;
    // This just to tell the iced what is the state we want out application to start with
    fn new(_flag: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                content: text_editor::Content::new(),
                error: None,
                path: None,
            },
            Command::perform(load_file(default_path()), Message::FileOpened),
        )
    }

    fn title(&self) -> String {
        String::from("notion")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Edit(action) => {
                self.content.edit(action);
                Command::none()
            }
            Message::NewFile => {
                self.content = text_editor::Content::new();
                self.path = None;
                Command::none()
            }
            Message::FileOpened(restuls) => match restuls {
                Ok(content) => {
                    self.path = Some(content.0);
                    self.content = text_editor::Content::with(&content.1);
                    Command::none()
                }
                Err(e) => {
                    self.error = Some(e);
                    Command::none()
                }
            },
            Message::OpenFile => Command::perform(pick_file(), Message::FileOpened),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // Controls bar with buttons
        let controls = row![
            button("open file").on_press(Message::OpenFile),
            button("New File").on_press(Message::NewFile)
        ]
        .spacing(10);

        // Text editor input
        let input = text_editor(&self.content).on_edit(Message::Edit);

        // File path or error display
        let file_path = if let Some(Error::IO(err)) = self.error.as_ref() {
            text(err.to_string())
        } else {
            match self.path.as_deref().and_then(Path::to_str) {
                Some(path) => text(path).size(14),
                None => text("New File"),
            }
        };

        // Line indicator showing cursor position
        let line_indicator = {
            let (line, col) = self.content.cursor_position();
            text(format!("{}:{}", line + 1, col + 1))
        };

        // Bottom bar layout
        let bottom_bar = row![file_path, horizontal_space(Length::Fill), line_indicator];

        // Overall layout container
        container(column![controls, input, bottom_bar].spacing(10))
            .padding(10)
            .into()
    }
    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

// So here the function prameter which sasys that it accept anything that inplments the
// AsRef<Path> trail,
// This is not the greatest. !!!
async fn load_file(path: impl AsRef<Path>) -> Result<(PathBuf, Arc<String>), Error> {
    let path = path.as_ref();
    let content = tokio::fs::read_to_string(path)
        .await
        .map(Arc::new)
        .map_err(|error| error.kind())
        .map_err(Error::IO)?;
    let path = path.to_path_buf();
    Ok((path, content))
}

async fn pick_file() -> Result<(PathBuf, Arc<String>), Error> {
    let handler = rfd::AsyncFileDialog::new()
        .set_title("Choose a text file it better be a .rs file")
        .pick_file()
        .await
        .ok_or(Error::CloseDigalog)?;
    load_file(handler.path().to_owned()).await
}
fn default_path() -> PathBuf {
    PathBuf::from(format!("{}/src/mai .rs", env!("CARGO_MANIFEST_DIR")))
}
#[derive(Debug, Clone)]
enum Error {
    CloseDigalog,
    IO(io::ErrorKind),
}
