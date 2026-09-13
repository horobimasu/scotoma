const sidebar = document.querySelector('.sidebar');
const main = document.querySelector('.main');

document.addEventListener('DOMContentLoaded', async () => {
    const fetched = await fetch('layout.json');
    const settings = await fetched.json();

    settings.toggles.forEach((config) => {
        const item = document.createElement('div');
        item.className = 'checkbox';

        const id = config.name
            .toLowerCase()
            .replace(/[^a-z\s]/g, '')
            .replaceAll(' ', '_');

        const checkbox = document.createElement('input');
        checkbox.type = 'checkbox';
        checkbox.checked = config.value || '';
        checkbox.id = id || 'noid';

        const label = document.createElement('label');
        label.textContent = config.name || 'you forgot to add a name';

        item.appendChild(checkbox);
        item.appendChild(label);
        sidebar.appendChild(item);
    });

    settings.main.forEach((config) => {
        const group = document.createElement('div');
        group.className = 'group';

        const label = document.createElement('span');
        label.className = 'label';
        label.textContent = config.name;

        let input = undefined;
        if (config.textarea) {
            input = document.createElement('textarea');
            input.style.height = config.height || '150px';
            group.style.alignItems = 'flex-end';
        } else {
            input = document.createElement('input');
            input.type = 'text';
        }

        const id = config.name
            .toLowerCase()
            .replace(/[^a-z\s]/g, '')
            .replaceAll(' ', '_');

        input.value = config.value || '';
        input.id = id || 'noid';

        if (config.numbers) {
            input.setAttribute('type', 'number');
        }

        const button = document.createElement('button');
        button.textContent = config.button || 'you forgot to add a name';
        button.id = 'gen-' + id;

        if (config.button === 'use default') {
            button.addEventListener('click', () => {
                input.style.border = '1px solid #ffffff';
                input.value = config.value || '';
            });
        } else if (config.button === 'gen random') {
            button.addEventListener('click', () => {
                input.style.border = '1px solid #ffffff';
                input.value = crypto.randomUUID();
            });
        }

        group.appendChild(label);
        group.appendChild(input);
        group.appendChild(button);

        main.insertBefore(group, buildButton);
    });

    const savedNote = localStorage.getItem('scotoma.note');
    if (savedNote) {
        const note = document.getElementById('note');
        note.value = savedNote;
    }
});

const getSettings = () => {
    const checkbox = document.querySelectorAll('checkbox');
    const inputs = document.querySelectorAll('input');
    const textareas = document.querySelectorAll('textarea');

    const settings = {};

    [...checkbox, ...inputs, ...textareas].forEach((item) => {
        if (!item.id) {
            return;
        }

        if (item.type === 'checkbox') {
            settings[item.id] = item.checked;
        } else {
            settings[item.id] = item.value;
        }
    });

    return settings;
};
