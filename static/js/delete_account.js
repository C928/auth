window.onload = async () => {
    const url = new URL(window.location.href);
    if (url.pathname === "/delete-account/cancel") {
        const token =  url.searchParams.get("token");
        if (!isValidURLTokenFmt(token)) {
            displayAPIResult("Invalid email verification link");
            return;
        }

        await sendCancelDeletionForm(token);
    } else {
        const deleteForm = document.getElementById('delete-form');
        deleteForm.addEventListener('submit', async (event) => {
            event.preventDefault();
            const deleteFormData = new FormData(deleteForm);
            if (!validateDeleteForm(deleteFormData)) {
                return;
            }

            await sendDeleteForm(deleteFormData);
        });
    }
};

async function sendCancelDeletionForm(token) {
    const resp = await fetch('/api/v1/user/delete/cancel?token=' + token);
    if (resp.ok) {
        const location = resp.headers.get("LOCATION");
        if (location) {
            sessionStorage.setItem("delete_msg", "Your account deletion request has been successfully canceled");
            window.location.href = window.location.origin + location;
            return;
        }
    } else if (resp.status === 400) {
        const json = await resp.json();
        displayAPIError(json, "account deletion cancellation");
        return;
    }

    console.error(resp);
    displayAPIResult("Something went wrong during account deletion cancellation");
}

async function sendDeleteForm(deleteForm) {
    if (!validateDeleteForm(deleteForm)) {
        return;
    }

    const updateFormEncoded = new URLSearchParams(deleteForm).toString();
    const resp = await fetch('/api/v1/user/delete/request', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/x-www-form-urlencoded',
        },
        body: updateFormEncoded,
    });

    if (resp.ok) {
        const location = resp.headers.get("LOCATION");
        if (location) {
            sessionStorage.setItem("delete_msg", "Your account has been setup for deletion. In 15 days all of its" +
                " associated data will be removed. To cancel the operation, log in to your account or click" +
                " on the link sent to you by email.");
            window.location.href = window.location.origin + location;
            return;
        }
    } else if (resp.status === 400) {
        const json = await resp.json();
        displayAPIError(json, "account deletion");
        return;
    }

    console.error(resp);
    displayAPIResult("Something went wrong during account deletion");
}

function validateDeleteForm(deleteForm) {
    const fieldNames = [
        "password",
        "confirmation_sentence"
    ];
    let fields = [];
    for (let i = 0; i < 2; i++) {
        const f = deleteForm.get(fieldNames[i]);
        if (f === null) {
            console.error("failed retrieving html input fields");
            return false;
        }
        fields[i] = f;
    }

    if (!isValidPasswordFmt(fields[0])) {
        displayAPIResult("Invalid password format");
        return false;
    }

    if (fields[1] !== "Delete my account.") {
        displayAPIResult("Confirmation sentence does not match");
        return false;
    }

    return true;
}
