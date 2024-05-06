/// validate password strength using zxcvbn
///
/// returns Some(String) if password is weak. The String contains the error message that can be
/// sent to the client
///
/// arg: (String, String, String)
/// arg.0: username
/// arg.1: email
pub fn is_weak_password(password: &str, arg: (&str, &str)) -> Option<String> {
    let entropy = match zxcvbn::zxcvbn(password, &[arg.0, arg.1, arg.0]) {
        Ok(entropy) => entropy,
        Err(e) => match e {
            zxcvbn::ZxcvbnError::BlankPassword => return Some("password is blank".to_string()),
            zxcvbn::ZxcvbnError::DurationOutOfRange => {
                return Some("password is too long".to_string())
            }
        },
    };

    // from zxcvbn docs:
    // Overall strength score from 0-4. Any score less than 3 should be considered too weak.
    if entropy.score() < 3 {
        let mut message = "password is too weak".to_string();

        if let Some(feedback) = entropy.feedback() {
            if let Some(warning) = feedback.warning() {
                message.push_str(format!("{}", warning).as_str());
            }

            feedback.suggestions().iter().for_each(|suggestion| {
                message.push_str(format!("{}", suggestion).as_str());
            });
        }

        return Some(message);
    }

    None
}
