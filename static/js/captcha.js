async function loadCaptcha() {
    let elems = getCaptchaElements(true);
    if (!elems) {
       return;
    }

    let id = elems["captcha-id"].value;
    if (id) {
        return;
    }

    const response = await fetch("/api/v1/captcha");
    if (!response.ok) {
        displayAPIResult("Couldn't load captcha image/audio");
        elems["captcha-loading"].remove();
        return;
    }

    await populateCaptchaElements(response, elems, true);
}

async function reloadCaptcha() {
    let elems = getCaptchaElements(false);
    if (!elems) {
        return;
    }

    let id = elems["captcha-id"].value;
    if (isValidCaptchaIDFmt(id)) {
        const response = await fetch("/api/v1/captcha/reload?id=" + id)
        if (!response.ok) {
            displayAPIResult("Couldn't reload captcha image/audio");
            return;
        }

        await populateCaptchaElements(response, elems, false);
    } else {
        console.error("Failed loading captcha html elements");
    }
}

function getCaptchaElements(isLoad) {
    let elems = {};
    const elementNames = ["captcha-id", "captcha-img"]; //,"captcha-wav"];
    if (isLoad) {
        elementNames.push("captcha-loading");
    }

    for (const e of elementNames) {
        const element = document.getElementById(e);
        if (element) {
//            element.value = "";
            elems[e] = element;
        } else {
            console.error("Failed loading captcha html elements");
            return;
        }
    }

    return elems;
}

async function populateCaptchaElements(response, elems, isLoad) {
    const json = await response.json();
    if (json) {
        const captchaJSON = JSON.stringify(json);
        //todo: parse in id/img/wav format (typed) (captcha.img /!\)
        const captcha = JSON.parse(captchaJSON);
        if (isLoad) {
            elems["captcha-loading"].remove();
        }
        elems["captcha-id"].value = captcha.id;
        elems["captcha-img"].style.display = "";
        elems["captcha-img"].src = "data:image/png;base64," + captcha.img;
        /*
        elems["captcha-wav"].style.display = "";
        elems["captcha-wav"].src = "data:audio/wav;base64," + captcha.wav;
         */
    }
}