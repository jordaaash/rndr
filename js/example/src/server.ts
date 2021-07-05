import express from 'express';
import { disburseFunds, fundJob } from './actions';
import { PORT } from './config';
import { getExplorerLink } from './util';

const app = express();

app.get('/', async (req, res) => {
    res.send(`
        <html>
            <body>
                <h2>Actions</h2>
                <ul>
                    <li><a href="/fund-job">Fund Job</a></li>
                    <li><a href="/disburse-funds">Disburse Funds</a></li>
                </ul>
            </body>
        </html>
    `);
});

app.get('/fund-job', async (req, res) => {
    const signature = await fundJob(1000000000); // 1 token
    const link = getExplorerLink(signature);

    res.send(`
        <html>
            <body>
                <h2>Job funded!</h2>
                <a href="${link}">${link}</a>
            </body>
        </html>
    `);
});

app.get('/disburse-funds', async (req, res) => {
    const signature = await disburseFunds(1000000000); // 1 token
    const link = getExplorerLink(signature);

    res.send(`
        <html>
            <body>
                <h2>Funds disbursed!</h2>
                <a href="${link}">${link}</a>
            </body>
        </html>
    `);
});

app.listen(PORT, () => {
    console.log(`Example app listening at http://localhost:${PORT}`);
});
