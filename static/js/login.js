window.onload = async () => {
    const account_deletion = sessionStorage.getItem("delete_msg");
    if (account_deletion != null) {
        displayAPIResult(account_deletion);
        sessionStorage.clear();
    }

    const loginForm = document.getElementById('login-form');
    const cancelBtn = document.getElementById("cancel-delete-btn");
    loginForm.addEventListener('submit', async (event) => {
        event.preventDefault();
        await sendLoginForm(loginForm, cancelBtn, false);
    });

    cancelBtn.onclick = async () => {
        await sendLoginForm(loginForm, cancelBtn, true);
    }
}

async function sendLoginForm(loginForm, cancelBtn, isCancelForm) {
    const loginFormData = new FormData(loginForm);
    loginFormData.append("cancel_deletion", isCancelForm ? "true" : "false");
    console.log("login form: ", loginFormData);
    if (!validateLoginForm(loginFormData)) {
        return;
    }

    const loginFormEncoded = new URLSearchParams(loginFormData).toString();
    console.log("string: ", loginFormEncoded);
    const resp = await fetch('/api/v1/user/login', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/x-www-form-urlencoded',
        },
        body: loginFormEncoded,
    });

    if (resp.status === 200) {
        const location = resp.headers.get("LOCATION");
        if (location) {
            window.location.href = window.location.origin + location;
            cancelBtn.style.display = "none";
            return;
        }
    } else if (resp.status === 409) {
        cancelBtn.style.display = "";
        return;
    } else if (resp.status === 400) {
        const json = await resp.json();
        displayAPIError(json, "authentication");
        return;
    }

    console.error(resp);
    displayAPIResult("Something went wrong during authentication");
}

function validateLoginForm(loginFormData) {
    const fieldNames = ["email", "password"];
    let fields = [];
    for (let i = 0; i < 2; i++) {
        const f = loginFormData.get(fieldNames[i]);
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

    if (!isValidPasswordFmt(fields[1])) {
        displayAPIResult("Invalid password format");
        return false;
    }

    return true;
}