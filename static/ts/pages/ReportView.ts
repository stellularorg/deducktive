const error: HTMLElement = document.getElementById("error")!;
const success: HTMLElement = document.getElementById("success")!;

const resolve_button: HTMLButtonElement | null = document.getElementById(
    "mark-as-resolved",
) as HTMLButtonElement | null;

if (resolve_button) {
     resolve_button.addEventListener("click", async (e) => {
        e.preventDefault();
        const res = await fetch(resolve_button.getAttribute("data-endpoint")!, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                status: "Archived",
            }),
        });

        const json = await res.json();

        if (json.success === false) {
            error.style.display = "block";
            error.innerHTML = `<div class="mdnote-title">${json.message}</div>`;
        } else {
            success.style.display = "block";
            success.innerHTML = `<div class="mdnote-title">${json.message}</div>`;
        }
    });
}

const spam_button: HTMLButtonElement | null = document.getElementById(
    "mark-as-spam",
) as HTMLButtonElement | null;

if (spam_button) {
     spam_button.addEventListener("click", async (e) => {
        e.preventDefault();
        const res = await fetch(spam_button.getAttribute("data-endpoint")!, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                status: "Spam",
            }),
        });

        const json = await res.json();

        if (json.success === false) {
            error.style.display = "block";
            error.innerHTML = `<div class="mdnote-title">${json.message}</div>`;
        } else {
            success.style.display = "block";
            success.innerHTML = `<div class="mdnote-title">${json.message}</div>`;
        }
    });
}
