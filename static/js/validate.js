function isValidEmailFmt(email) {
    //todo
    return true;
}

function isValidUsernameFmt(username) {
    const len = username.length;
    const validChars = /^[a-zA-Z0-9_.]+$/;
    return len > 1 && len < 31 && validChars.test(username);
}

function isValidPasswordFmt(password) {
    const len = password.length;
    if (len < 8 || len > 100) {
        return false;
    }

    const valid = [false, false, false, false];
    for (const c of password) {
        if (c.match(/[a-z]/)) {
            valid[0] = true;
        } else if (c.match(/[A-Z]/)) {
            valid[1] = true;
        } else if (c.match(/\d/)) {
            valid[2] = true;
        } else if (c.match(/[!@#$%^&*()_+{}\[\]:;<>,.?~\\/-]/)) {
            valid[3] = true;
        }
    }

    return !valid.includes(false);
}

function passwordConfirmMatchPassword(newPassword, newPasswordConfirm) {
    return newPassword === newPasswordConfirm;
}

function isValidCaptchaAnswerFmt(answer) {
    const len = answer.length;
    const validChars = /^[a-zA-Z0-9]+$/;
    return len > 3 && len < 7 && validChars.test(answer);
}

function isValidCaptchaIDFmt(id) {
    const regex = /^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-4[0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$/;
    return regex.test(id);
}

function isValidURLTokenFmt(token) {
    const len = token.length;
    const validChars = /^[a-zA-Z0-9]+$/;
    return len === 150 && validChars.test(token);
}