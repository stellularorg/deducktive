const report_form = document.getElementById(
    "report_page",
) as HTMLFormElement | null;

const warning = document.getElementById(
    "warning",
) as HTMLDivElement | null;

if (window.top && report_form && warning) {
    document.getElementById("continue")!.addEventListener("click", () => {
        report_form.style.display = "flex";
        warning.remove();
    });
    
    report_form.addEventListener("submit", async (event) => {
        event.preventDefault();

        const res = await fetch("/api/v1/reports", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                report_type:
                    report_form.report_type.options[
                        report_form.report_type.selectedIndex
                    ].value,
                content: report_form.content.value,
                address: window.location.href,
                // get current user username
                as_user: (window as any).REPORT_AS_USER,
            }),
        });

        const json = await res.json();
        document.body.innerHTML = `<p>${json.message} -- Please exit this form.</p>`;
    });
}
