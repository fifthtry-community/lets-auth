function show_alert() {
    let message = "";
    for (let i = 0; i < arguments.length; i++) {
        let argument = arguments[i]
        if (argument instanceof fastn.recordInstanceClass) {
            let error = argument.get("error").get();
            if (error) {
                error = " [error: " + error + "]";
            } else {
                error = " [no error]";
            }
            message += (
                argument.get("name").get() + ": " + argument.get("value").get() + error + "\n"
            );
        } else {
            message += argument + "\n";
        }
    }

    alert(message);
}
