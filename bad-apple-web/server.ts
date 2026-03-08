import express from 'express';
import cors from 'cors';
import youtubeDl from 'youtube-dl-exec';

const app = express();
app.use(cors());
app.use(express.json());

app.post('/api/get-stream', async (req, res) => {
    const { url } = req.body;
    if (!url) return res.status(400).json({ error: 'URL is required' });

    try {
        console.log(`Extracting stream URL for: ${url}`);
        const result = await youtubeDl(url, {
            dumpSingleJson: true,
            noWarnings: true,
            noCheckCertificates: true,
            preferFreeFormats: true,
            youtubeSkipDashManifest: true,
        });

        const rawResult: any = result;

        // Try to find an MP4 video format
        const format = rawResult.formats.find(
            (f: any) => f.ext === 'mp4' && f.vcodec !== 'none' && f.acodec !== 'none'
        ) || rawResult.formats.find(
            (f: any) => f.vcodec !== 'none'
        );

        if (!format || !format.url) {
            throw new Error('No valid video stream URL found');
        }

        res.json({ streamUrl: format.url, title: rawResult.title });
    } catch (error: any) {
        console.error('Error extracting URL:', error.message);
        res.status(500).json({ error: 'Failed to extract stream URL', details: error.message });
    }
});

const PORT = 3001;
app.listen(PORT, () => {
    console.log(`Server running at http://localhost:${PORT}`);
});
