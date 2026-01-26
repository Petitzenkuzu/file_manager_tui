use crossterm::event::KeyCode;
use crate::file_manager::FileManagerAction;
use crate::app::App;
use crossterm::event::Event;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
impl App {

    /// Handles the inputs when the focus is on the files list with filter mode on
    pub fn handle_filter_input(&mut self, event: Event) {
        if let Event::Key(KeyEvent { code, kind: KeyEventKind::Press, modifiers, .. }) = event {
            if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('f') {
                self.filter_mode = !self.filter_mode;
                self.filter_buffer.clear();
                self.list_state.select(self.min_selected());
                let _ = self.file_manager.dispatch(FileManagerAction::ReadContent(self.min_selected()));
                return;
            }
            match code {
                KeyCode::Up => {
                    if self.filtered_files.is_empty() {
                        return;
                    }
                    let mut selected = match self.list_state.selected() {
                        Some(selected) => selected,
                        None => {
                            if let Some(min) = self.min_filtered_selected() {
                                self.list_state.select(Some(min));
                                let _ = self.file_manager.dispatch(FileManagerAction::ReadContent(Some(self.filtered_files[min])));
                            }
                            return;
                        },
                    };
                    // handle the selection index underflow
                    if modifiers.contains(KeyModifiers::CONTROL) {
                        let (res, overflow) = selected.overflowing_sub(5);
                        if overflow {
                            let overflow_len = (usize::MAX - res).strict_rem(self.filtered_files.len()) ;
                            selected = res.clamp(0, self.filtered_files.len()- (1 + overflow_len));
                        }
                        else {
                            selected = res;
                        }
                    }
                    else {
                        let (res, overflow) = selected.overflowing_sub(1);
                        if overflow {
                            selected = self.filtered_files.len() - 1;
                        }
                        else {
                            selected = res;
                        }
                    }
                    self.list_state.select(Some(selected));
                    let _ = self.file_manager.dispatch(FileManagerAction::ReadContent(Some(self.filtered_files[selected])));
                },
                KeyCode::Down => {
                    if self.filtered_files.is_empty() {
                        return;
                    }
                    let mut selected = match self.list_state.selected() {
                        Some(selected) => selected,
                        None => {
                            if let Some(min) = self.min_filtered_selected() {
                                self.list_state.select(Some(min));
                                let _ = self.file_manager.dispatch(FileManagerAction::ReadContent(Some(self.filtered_files[min])));
                            }
                            return;
                        },
                    };
                    // handle the selection 
                    if modifiers.contains(KeyModifiers::CONTROL) {
                        selected = selected + 5;
                        selected = selected.strict_rem(self.filtered_files.len());
                    }
                    else {
                        selected = selected + 1;
                        selected = selected.strict_rem(self.filtered_files.len());
                    }
                    self.list_state.select(Some(selected));
                    let _ = self.file_manager.dispatch(FileManagerAction::ReadContent(Some(self.filtered_files[selected])));
                },
                KeyCode::Char(c) => {
                    // push the new character to the filter buffer then update the ui 
                    self.filter_buffer.push(c);
                    self.update_filtered_files();
                    let min = self.min_filtered_selected();
                    self.list_state.select(min);
                    if let Some(min) = min {
                        let _ = self.file_manager.dispatch(FileManagerAction::ReadContent(Some(self.filtered_files[min])));
                    }
                },
                KeyCode::Backspace => {
                    // pop the last character from the filter buffer then update the ui 
                    self.filter_buffer.pop();
                    self.update_filtered_files();
                    let min = self.min_filtered_selected();
                    self.list_state.select(min);
                    if let Some(min) = min {
                        let _ = self.file_manager.dispatch(FileManagerAction::ReadContent(Some(self.filtered_files[min])));
                    }
                },
                KeyCode::Enter => {
                    let selected = match self.list_state.selected() {
                        Some(selected) => selected,
                        None => return,
                    };
                    let _ = self.file_manager.dispatch(FileManagerAction::Open(self.filtered_files[selected]));
                },
                _ => {}
            }
        }
    }


    /// Handles the inputs when the focus is on the files list with filter mode off
    pub fn handle_files_input(&mut self, event: Event) {
        if let Event::Key(KeyEvent { code, kind: KeyEventKind::Press, modifiers, .. }) = event {
            match code {
                KeyCode::Up => {
                    if self.file_manager.files().is_empty() {
                        return;
                    }
                    let mut selected = match self.list_state.selected() {
                        Some(selected) => selected,
                        None => {
                            self.list_state.select(self.min_selected());
                            let _ = self.file_manager.dispatch(FileManagerAction::ReadContent(self.min_selected()));
                            return;
                        },
                    };
                    if modifiers.contains(KeyModifiers::CONTROL) {
                        let (res, overflow) = selected.overflowing_sub(5);
                        if overflow {
                            let overflow_len = (usize::MAX - res).strict_rem(self.file_manager.files().len()) ;
                            selected = res.clamp(0, self.file_manager.files().len()- (1 + overflow_len));
                        }
                        else {
                            selected = res;
                        }
                    }
                    else {
                        let (res, overflow) = selected.overflowing_sub(1);
                        if overflow {
                            selected = self.file_manager.files().len() - 1;
                        }
                        else {
                            selected = res;
                        }
                    }
                    self.list_state.select(Some(selected));
                    let _ = self.file_manager.dispatch(FileManagerAction::ReadContent(self.list_state.selected()));
                },
                KeyCode::Down => {
                    if self.file_manager.files().is_empty() {
                        return;
                    }
                    let mut selected = match self.list_state.selected() {
                        Some(selected) => selected,
                        None => {
                            self.list_state.select(self.min_selected());
                            let _ = self.file_manager.dispatch(FileManagerAction::ReadContent(self.min_selected()));
                            return;
                        },
                    };

                    if modifiers.contains(KeyModifiers::CONTROL) {
                        selected = selected + 5;
                        selected = selected.strict_rem(self.file_manager.files().len());
                    }
                    else {
                        selected = selected + 1;
                        selected = selected.strict_rem(self.file_manager.files().len());
                    }
                    self.list_state.select(Some(selected));
                    let _ = self.file_manager.dispatch(FileManagerAction::ReadContent(self.list_state.selected()));
                },
                KeyCode::Enter => {
                    let selected = match self.list_state.selected() {
                        Some(selected) => selected,
                        None => return,
                    };
                    let _ = self.file_manager.dispatch(FileManagerAction::Open(selected));
                },
                KeyCode::Backspace => {
                    let _ = self.file_manager.dispatch(FileManagerAction::GoToParent);
                },
                KeyCode::F(5) => {
                    let _ = self.file_manager.dispatch(FileManagerAction::Reload);
                },
                KeyCode::Char('f') => {
                    if modifiers.contains(KeyModifiers::CONTROL) {
                        self.filter_mode = !self.filter_mode;
                        self.filter_buffer.clear();
                        self.update_filtered_files();
                        // for convenience, we do not touch to the selection index cause if it's empty it stays empty and if it's different from None then it's still valid cause default filter is empty
                    }
                },
                _ => {}
            }
        }
    }
}