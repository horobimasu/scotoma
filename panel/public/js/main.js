let canBuild = true;
let failedChecks = [];

const mustGen = [
    'tracking_id',
    'note',
    'files_enumeration_chunk_size',
    'overwrite_deleted_data_passes'
];

const get = (id, element) => {
    return element ? document.getElementById(id) : document.getElementById(id).value.trim();
};

const request = async (url, body) => {
    const res = await fetch(url, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body)
    });

    if (!res.ok) {
        const text = await res.text();
        alert(`the request is not ok - ${text}`);

        window.location.href = window.location.pathname;
        return;
    }

    return res;
};

buildButton.addEventListener('click', async () => {
    if (!canBuild) {
        return;
    }

    canBuild = false;

    mustGen.forEach((item) => {
        if (get(item) !== '') {
            return;
        }

        failedChecks.push(item);
    });

    if (failedChecks.length !== 0) {
        failedChecks.forEach((id) => {
            get(id, true).style.border = '1px solid #ff0000';
        });

        alert('one or more mandatory settings have not been configured');

        failedChecks = [];
        canBuild = true;

        return;
    }

    localStorage.setItem('scotoma.note', note.value);

    document.title = document.title + ' - building...';

    const settings = getSettings();
    await request('/api/update-config', settings);

    const build = await request('/api/start-build');

    window.location.href = window.location.pathname + `?buildsuc=${build.status === 200 ? 'yes' : 'no'}`;
});
