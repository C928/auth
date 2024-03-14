function displayAPIResult(result) {
    const apiResultDiv = document.getElementById("api-result");
    if (!apiResultDiv) {
        console.error("Failed retrieving api-result div");
    }

    apiResultDiv.textContent = result;
}

function displayAPIError(json, actionType) {
    const error = Object.keys(json)[0];
    if (error === "unknown") {
        displayAPIError("Unexpected error happened during " + actionType);
        return;
    }

    if (error === "session_error") {
        console.error("Error type: session app_error | Error description", json[error]);
        window.location.href = window.location.origin + "/login";
    }

    const errorDescription = json[error];
    console.error("Error type:", error, "| Error description:", errorDescription);
    displayAPIResult(errorDescription.replaceAll("_", " "));
}