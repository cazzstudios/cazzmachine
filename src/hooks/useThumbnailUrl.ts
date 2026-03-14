import { useState, useEffect } from 'react';
import { fetchImage } from '../lib/tauri';

interface UseThumbnailUrlOptions {
  onError?: 'silent' | 'null';
}

export function useThumbnailUrl(
  thumbnailData: string | null | undefined,
  thumbnailUrl: string | null | undefined,
  options: UseThumbnailUrlOptions = { onError: 'null' }
): string | null {
  const [imageUrl, setImageUrl] = useState<string | null>(thumbnailData || null);

  useEffect(() => {
    setImageUrl(null);

    if (thumbnailData) {
      setImageUrl(thumbnailData);
    } else if (thumbnailUrl) {
      let url = thumbnailUrl;
      if (url.startsWith('//')) {
        url = 'https:' + url;
      }
      if (url.startsWith('http://') || url.startsWith('https://')) {
        fetchImage(url)
          .then(setImageUrl)
          .catch(() => {
            if (options.onError === 'null') {
              setImageUrl(null);
            }
            // If 'silent', do nothing (image fails silently)
          });
      } else {
        setImageUrl(url);
      }
    }
  }, [thumbnailData, thumbnailUrl, options.onError]);

  return imageUrl;
}
