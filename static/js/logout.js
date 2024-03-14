async function logout() {
    sessionStorage.clear();
    await fetch('/api/v1/user/logout');
    window.location.href = window.location.origin + "/login";
}