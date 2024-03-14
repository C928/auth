window.onload = async () => {
    const cachedUserData = await getSessionUserData();
    if (cachedUserData) {
        fillFormInputs(cachedUserData);
    }

    const updateForm = document.getElementById('update-form');
    updateForm.addEventListener('submit', async (event) => {
        event.preventDefault();
        let updateFormData = new FormData(updateForm);
        await sendUpdateForm(updateFormData, cachedUserData);
    });

    const deleteForm = document.getElementById('delete-form');
    deleteForm.addEventListener('submit', async (event) => {
        event.preventDefault();
        let deleteFormData = new FormData(deleteForm);
        await sendDeleteForm(deleteFormData);
    });
}

async function getSessionUserData() {
    const sessionUserData = sessionStorage.getItem('userData');
    if (sessionUserData) {
        return JSON.parse(sessionUserData);
    } else {
        const resp = await fetch('/api/v1/user/data');
        if (resp.status === 400) {
            const json = await resp.json();
            displayAPIError(json, "update of credentials");
            return;
        }

        const userData = await resp.json();
        if (userData) {
            sessionStorage.setItem('userData', JSON.stringify(userData));
            return userData;
        }
    }
}

function fillFormInputs(cachedUserData) {
    const newEmail = document.getElementById('new-email');
    const newUsername = document.getElementById('new-username');
    if (newEmail && newUsername) {
        newEmail.value = cachedUserData.email;
        newUsername.value = cachedUserData.username;
    } else {
        console.error("Failed loading html elements");
    }
}

// newUserData must be in this format: [['email', 'email@example.com'], ['username', 'some_username']]
function updateSessionUserData(newUserData) {
    let userData = sessionStorage.getItem('userData');
    if (!userData) {
        return;
    }
    userData = JSON.parse(userData);

    for (const [key, value] of newUserData) {
        userData[key] = value;
    }

    userData = JSON.stringify(userData);
    sessionStorage.setItem('userData', userData);
}

async function sendUpdateForm(updateForm, cachedUserData) {
    let newCacheData = [];
    if (!validateUpdateForm(updateForm, cachedUserData, newCacheData)) {
        return;
    }

    const updateFormEncoded = new URLSearchParams(updateForm).toString();
    const resp = await fetch('/api/v1/user/update', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/x-www-form-urlencoded',
        },
        body: updateFormEncoded,
    });

    if (resp.ok) {
        displayAPIResult("Account details successfully updated");
        const elements = [
            "new-password",
            "new-password-confirm",
            "update-form-password",
            "update-form-confirm-sentence",
            "delete-form-password",
            "delete-form-confirm-sentence"
        ];
        for (const e of elements) {
            const element = document.getElementById(e);
            if (element) {
                element.value = "";
            } else {
                console.error("Failed loading html elements");
                return;
            }
        }

        updateSessionUserData(newCacheData);
    } else if (resp.status === 400) {
        const json = await resp.json();
        displayAPIError(json, "update of credentials");
    }
}

function validateUpdateForm(updateForm, cachedUserData, newCacheData) {
    const fieldNames = [
        "new_email",
        "new_username",
        "new_password",
        "new_password_confirm",
        "password",
        "confirmation_sentence"
    ];
    let fields = [];
    for (let i = 0; i < 6; i++) {
        const f = updateForm.get(fieldNames[i]);
        if (f === null) {
            console.error("failed retrieving html input fields");
            return false;
        }
        fields[i] = f;
    }

    if (fieldNames[0] === cachedUserData.email) {
        updateForm.set('new_email', '');
    } else if (!isValidEmailFmt(fields[0])) {
        displayAPIResult("Invalid new email format");
        return false;
    } else {
        newCacheData.push(['email', fields[0]]);
    }

    if (fields[1] === cachedUserData.username) {
        updateForm.set('new_username', '');
    } else if (!isValidUsernameFmt(fields[1])) {
        displayAPIResult("Invalid new username format");
        return false;
    } else {
        newCacheData.push(['username', fields[1]]);
    }

    if (fields[2] || fields[3]) {
        if (!isValidPasswordFmt(fields[2])) {
            displayAPIResult("Invalid new password format");
            return false;
        }

        if (!passwordConfirmMatchPassword(fields[2], fields[3])) {
            displayAPIResult("New password confirm does not match new password");
            return false;
        }
    }

    if (!isValidPasswordFmt(fields[4])) {
        displayAPIResult("Invalid password format");
        return false;
    }

    if (fields[5] !== "Update my account.") {
        displayAPIResult("Confirmation sentence does not match");
        return false;
    }

    return true;
}