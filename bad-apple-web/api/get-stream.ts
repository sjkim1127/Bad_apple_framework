import type { VercelRequest, VercelResponse } from '@vercel/node';
// @ts-ignore
import youtubeDl from 'youtube-dl-exec';

export default async function handler(
    req: VercelRequest,
    res: VercelResponse
) {
    // Add CORS headers for testing
    res.setHeader('Access-Control-Allow-Credentials', 'true');
    res.setHeader('Access-Control-Allow-Origin', '*');
    res.setHeader('Access-Control-Allow-Methods', 'GET,OPTIONS,PATCH,DELETE,POST,PUT');
    res.setHeader(
        'Access-Control-Allow-Headers',
        'X-CSRF-Token, X-Requested-With, Accept, Accept-Version, Content-Length, Content-MD5, Content-Type, Date, X-Api-Version'
    );

    if (req.method === 'OPTIONS') {
        res.status(200).end();
        return;
    }

    if (req.method !== 'POST') {
        return res.status(405).json({ error: 'Method not allowed' });
    }

    const { url } = req.body;

    if (!url) {
        return res.status(400).json({ error: 'URL is required' });
    }

    try {
        console.log(`Extracting stream URL for: ${url}`);

        // Note: Vercel serverless functions have a 10s execution limit on free tier.
        // Fetching formats from YouTube might occasionally take longer, leading to timeouts.
        const result: any = await youtubeDl(url, {
            dumpSingleJson: true,
            noWarnings: true,
            noCheckCertificates: true,
            preferFreeFormats: true,
            youtubeSkipDashManifest: true,
        });

        // Try to find an MP4 video format
        const format = result.formats.find(
            (f: any) => f.ext === 'mp4' && f.vcodec !== 'none' && f.acodec !== 'none'
        ) || result.formats.find(
            (f: any) => f.vcodec !== 'none'
        );

        if (!format || !format.url) {
            throw new Error('No valid video stream URL found');
        }

        res.status(200).json({ streamUrl: format.url, title: result.title });
    } catch (error: any) {
        console.error('Error extracting URL:', error.message);
        res.status(500).json({ error: 'Failed to extract stream URL', details: error.message });
    }
}
