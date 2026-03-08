import express from 'express';
import cors from 'cors';
import ytdl from '@distube/ytdl-core';

const app = express();
app.use(cors());
app.use(express.json());

app.post('/api/get-stream', async (req, res) => {
    const { url } = req.body;
    if (!url) return res.status(400).json({ error: 'URL is required' });

    try {
        console.log(`Extracting stream URL for: ${url}`);

        const info = await ytdl.getInfo(url);

        // Find format with both video & audio, highest quality, or video only fallback
        const format = ytdl.chooseFormat(info.formats, { quality: 'highest' });

        if (!format || !format.url) {
            throw new Error('No valid video stream URL found');
        }

        res.json({ streamUrl: format.url, title: info.videoDetails.title });
    } catch (error: any) {
        console.error('Error extracting URL:', error.message);
        res.status(500).json({ error: 'Failed to extract stream URL', details: error.message });
    }
});

const PORT = 3001;
app.listen(PORT, () => {
    console.log(`Server running at http://localhost:${PORT}`);
});
