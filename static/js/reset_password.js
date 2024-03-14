window.onload = async () => {
    const resetPasswordRequestForm = document.getElementById('reset-password-request-form');
    if (resetPasswordRequestForm) {
        await loadCaptcha();
        resetPasswordRequestForm.addEventListener('submit', async (event) => {
            event.preventDefault();
            const resetPasswordRequestFD = new FormData(resetPasswordRequestForm);
            if (!validatePasswordResetRequestForm(resetPasswordRequestFD)) {
                return;
            }

            await sendPasswordResetRequestForm(resetPasswordRequestFD);
        });
    }

    const resetPasswordForm = document.getElementById('reset-password-form');
    if (resetPasswordForm) {
        resetPasswordForm.addEventListener('submit', async (event) => {
            event.preventDefault();
            const resetPasswordFD = new FormData(resetPasswordForm);
            const url = new URL(window.location.href)
            const urlParams = new URLSearchParams(url.search);
            const token = urlParams.get("token");
            resetPasswordFD.append("token", token);
            if (!validatePasswordResetForm(resetPasswordFD)) {
                return;
            }

            await sendPasswordResetForm(resetPasswordFD);
        });
    }
}

function validatePasswordResetRequestForm(passwordResetRequestForm) {
    const email = passwordResetRequestForm.get("email");
    if (email === null) {
        console.error("failed retrieving html input field (email)");
        return false;
    }

    if (!isValidEmailFmt(email)) {
        displayAPIResult("Invalid email format");
        return false;
    }

    return true;
}

function validatePasswordResetForm(passwordResetForm) {
    const fieldNames = ["token", "new_password", "new_password_confirm"];
    let fields = [];
    for (let i = 0; i < 3; i++) {
        const f = passwordResetForm.get(fieldNames[i]);
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

    if (!isValidPasswordFmt(fields[1])) {
        displayAPIResult("Invalid email format");
        return false;
    }

    if (!passwordConfirmMatchPassword(fields[1], fields[2])) {
        displayAPIResult("Password confirm does not match password");
        return false;
    }

    return true;
}

async function sendPasswordResetRequestForm(resetPasswordRequestForm) {
    const formEncoded = new URLSearchParams(resetPasswordRequestForm).toString();
    const resp = await fetch('/api/v1/reset-password/request', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/x-www-form-urlencoded',
        },
        body: formEncoded,
    });

    if (resp.ok) {
        displayAPIResult("An email has been sent to your address.\
            Please click on the link contained in this email verify your identity.")
        return;
    } else if (resp.status === 400) {
        const json = await resp.json();
        displayAPIError(json, "password reset request");
        return;
    }

    console.error(resp);
    displayAPIResult("Something went wrong during password reset request");
}

async function sendPasswordResetForm(resetPasswordForm) {
    const formEncoded = new URLSearchParams(resetPasswordForm).toString();
    const resp = await fetch('/api/v1/reset-password', {
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
        displayAPIError(json, "password reset");
        return;
    }

    console.error(resp);
    displayAPIResult("Something went wrong during password reset");
}