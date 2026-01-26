use crate::app::App;
use ratatui::{
    buffer::Buffer, layout::Rect, text::{Line, Text}, widgets::{Block, List, Padding, Paragraph, StatefulWidget, Widget}
};
use ratatui::layout::{Layout, Direction, Constraint};
use ratatui::style::{Style, Color};

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {

        // we shrink preview section to let space for files section if the terminal is too small
        let area_width = area.width;
        let files_section_width_constrain = if area_width / 2 > crate::app::MIN_FILES_SECTION_WIDTH { Constraint::Percentage(50) } else { Constraint::Length(crate::app::MIN_FILES_SECTION_WIDTH) };
        // split the screen into 2 parts horizontally left (Files) and right (File preview if available on selected file)
        let main_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                files_section_width_constrain,
                Constraint::Percentage(100),
            ])
            .split(area);

        let list_symbol = "->";
        // dynamically calculate the max name width based on the screen width and the modified time width
        self.max_name_width = std::cmp::max(crate::app::MIN_NAME_WIDTH, (main_layout[0].width as usize).saturating_sub(crate::app::MODIFIED_TIME_WIDTH+list_symbol.len()));

        // split the left part into 2 parts vertically top (Path) and bottom (File list)
        let files_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Percentage(100),
                Constraint::Length(1),
            ])
            .split(main_layout[0]);

        // render the file list
        let files_items = match self.filter_mode {
            true => self.filtered_files.iter().map(|index| self.file_manager.files()[*index].to_line(self.max_name_width-list_symbol.len(), crate::app::MODIFIED_TIME_WIDTH)).collect::<Vec<Line>>(),
            false => self.file_manager.files().iter().map(|file| file.to_line(self.max_name_width-list_symbol.len(), crate::app::MODIFIED_TIME_WIDTH)).collect::<Vec<Line>>(),
        };
        let list = List::new(files_items).block(Block::default()).highlight_symbol("->").repeat_highlight_symbol(true);
        StatefulWidget::render(list, files_layout[1], buf, &mut self.list_state);

        // render the path
        let _path_display = Paragraph::new(Text::from(self.file_manager.path().to_string_lossy().to_string())).block(Block::default().padding(Padding::new(1, 0, 1, 0))).left_aligned().render(files_layout[0], buf);

        let _preview_display = Paragraph::new(Text::from(self.file_manager.selected_file_preview_buffer())).block(Block::default().title(Line::from(" Preview ").centered())).render(main_layout[1], buf);

        let _filter_mode_display = Paragraph::new(Text::from(if self.filter_mode { format!("Applied filter: \"{}\"", self.filter_buffer) } else { "Filter mode OFF".to_string() }).style(Style::default().black())).style(Style::default().bg(Color::White)).left_aligned().render(files_layout[2], buf);
    
    }
}