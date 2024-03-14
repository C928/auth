window.onload = async () => {
    const registerRequestForm = document.getElementById('register-request-form');
    if (registerRequestForm) {
        await loadCaptcha();
        registerRequestForm.addEventListener('submit', async (event) => {
            event.preventDefault();
            let registerRequestFD = new FormData(registerRequestForm);
            if (!validateRegisterRequestForm(registerRequestFD)) {
                return;
            }

            await sendRegisterRequestForm(registerRequestFD);
        });
    }

    const registerForm = document.getElementById('register-form');
    if (registerForm) {
        registerForm.addEventListener('submit', async (event) => {
            event.preventDefault();
            let registerFD = new FormData(registerForm);
            const url = new URL(window.location.href)
            const urlParams = new URLSearchParams(url.search);
            const token = urlParams.get("token");
            registerFD.append("token", token);
            if (!validateRegisterForm(registerFD)) {
                return;
            }

            await sendRegisterForm(registerFD);
        });
    }
}

function validateRegisterRequestForm(registerRequestForm) {
    const fieldNames = ["email", "captcha_id", "captcha_answer"];
    let fields = [];
    for (let i = 0; i < 3; i++) {
        const f = registerRequestForm.get(fieldNames[i]);
        if (f === null) {
            console.error("failed retrieving html input fields");
            return false;
        }
        fields[i] = f;
    }

    if (!isValidEmailFmt(fields[0])) {
        displayAPIResult("Invalid email format");
        return false;
    }

    if (!isValidCaptchaIDFmt(fields[1])) {
        displayAPIResult("Failed loading captcha");
        return false;
    }

    if (!isValidCaptchaAnswerFmt(fields[2])) {
        displayAPIResult("invalid captcha answer");
        return false;
    }

    return true;
}

async function sendRegisterRequestForm(registerRequestForm) {
    const formEncoded = new URLSearchParams(registerRequestForm).toString();
    const resp = await fetch('/api/v1/user/create/request', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/x-www-form-urlencoded',
        },
        body: formEncoded,
    });

    if (resp.ok) {
        displayAPIResult("An email has been sent to your address.\
            Please click on the link contained in this email to confirm your account creation.")
        return;
    } else if (resp.status === 400) {
        const json = await resp.json();
        displayAPIError(json, "registration");
        return;
    }

    console.error(resp);
    displayAPIResult("Something went wrong during registration");
}

function validateRegisterForm(registerForm) {
    const fieldNames = ["token", "username", "password", "password_confirm"];
    let fields = [];
    for (let i = 0; i < 4; i++) {
        const f = registerForm.get(fieldNames[i]);
        if (f === null) {
            console.error("failed retrieving html input fields");
            return false;
        }
        fields[i] = f;
    }

    if (!isValidURLTokenFmt(fields[0])) {
        displayAPIResult("Invalid email confirmation link");
        return false;
    }

    if (!isValidUsernameFmt(fields[1])) {
        displayAPIResult("Invalid username format");
        return false;
    }

    if (!isValidPasswordFmt(fields[2])) {
        displayAPIResult("Invalid password format");
        return false;
    }

    if (!passwordConfirmMatchPassword(fields[2], fields[3])) {
        displayAPIResult("Password confirm does not match password");
        return false;
    }

    return true;
}

async function sendRegisterForm(registerForm) {
    const formEncoded = new URLSearchParams(registerForm).toString();
    const resp = await fetch('/api/v1/user/create', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/x-www-form-urlencoded',
        },
        body: formEncoded,
    });

    if (resp.ok) {
        const location = resp.headers.get("LOCATION");
        if (location) {
            window.location.href = window.location.origin + location;
            return;
        }
    } else if (resp.status === 400) {
        const json = await resp.json();
        displayAPIError(json, "registration");
        return;
    }

    console.error(resp);
    displayAPIResult("Something went wrong during registration");
}
