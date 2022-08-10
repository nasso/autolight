use win32_notification::NotificationBuilder;

pub fn notify(title: &str, body: &str) {
    NotificationBuilder::new()
        .title_text(title)
        .info_text(body)
        .build()
        .unwrap()
        .show()
        .unwrap();
}
