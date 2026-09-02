use ratatui::buffer::Buffer;

pub fn lines(buffer: &Buffer) -> Vec<String> {
    let area = buffer.area;

    (area.top()..area.bottom())
        .map(|y| {
            (area.left()..area.right())
                .map(|x| buffer[(x, y)].symbol())
                .collect()
        })
        .collect()
}
