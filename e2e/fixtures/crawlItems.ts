export interface CrawlItem {
  id: string;
  source: string;
  category: string;
  title: string;
  url: string;
  thumbnail_url: string | null;
  thumbnail_data: string | null;
  description: string | null;
  fetched_at: string;
  is_seen: boolean;
  is_saved: boolean;
  session_date: string;
}

const TINY_PNG_BASE64 = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAgAAAAICAYAAADED76LAAAAFklEQVQoU2NkYGD4z0AEYBw1YNQAJgBNSQEJWDQmhwAAAABJRU5ErkJggg==';

const today = new Date().toISOString().split('T')[0];
const now = new Date().toISOString();

export const TEST_ITEMS: CrawlItem[] = [
  {
    id: 'test-meme-001',
    source: 'test-reddit-memes',
    category: 'meme',
    title: 'Test Meme: When You Finally Fix That Bug',
    url: 'https://example.com/meme/001',
    thumbnail_url: null,
    thumbnail_data: TINY_PNG_BASE64,
    description: 'Classic programming meme',
    fetched_at: now,
    is_seen: false,
    is_saved: false,
    session_date: today,
  },
  {
    id: 'test-joke-001',
    source: 'test-dadjoke-api',
    category: 'joke',
    title: 'Test Joke',
    url: 'https://example.com/joke/001',
    thumbnail_url: null,
    thumbnail_data: null,
    description: 'Why do programmers prefer dark mode? Because light attracts bugs!',
    fetched_at: now,
    is_seen: false,
    is_saved: false,
    session_date: today,
  },
  {
    id: 'test-news-001',
    source: 'test-news-feed',
    category: 'news',
    title: 'Test News: Major Tech Breakthrough Announced',
    url: 'https://example.com/news/001',
    thumbnail_url: null,
    thumbnail_data: TINY_PNG_BASE64,
    description: 'Scientists discover new algorithm that makes tests run faster. Revolutionary breakthrough in software testing methodology.',
    fetched_at: now,
    is_seen: false,
    is_saved: false,
    session_date: today,
  },
  {
    id: 'test-video-001',
    source: 'test-video-platform',
    category: 'video',
    title: 'Test Video: How to Write Better Tests',
    url: 'https://example.com/video/001',
    thumbnail_url: null,
    thumbnail_data: TINY_PNG_BASE64,
    description: 'Educational video about test-driven development',
    fetched_at: now,
    is_seen: false,
    is_saved: false,
    session_date: today,
  },
  {
    id: 'test-gossip-001',
    source: 'test-gossip-feed',
    category: 'gossip',
    title: 'Test Celebrity Spotted Using New Framework',
    url: 'https://example.com/gossip/001',
    thumbnail_url: null,
    thumbnail_data: null,
    description: 'Exclusive: Famous developer seen contributing to open source',
    fetched_at: now,
    is_seen: false,
    is_saved: false,
    session_date: today,
  },
];

export const TEST_ITEMS_WITH_URLS: CrawlItem[] = [
  {
    id: 'test-meme-url-001',
    source: 'test-reddit-memes',
    category: 'meme',
    title: 'Test Meme: Valid Placeholder URL Thumbnail',
    url: 'https://example.com/meme/url-001',
    thumbnail_url: 'https://via.placeholder.com/150',
    thumbnail_data: null,
    description: 'Meme with valid HTTPS thumbnail URL',
    fetched_at: now,
    is_seen: false,
    is_saved: false,
    session_date: today,
  },
  {
    id: 'test-news-protocol-001',
    source: 'test-news-feed',
    category: 'news',
    title: 'Test News: Protocol-Relative URL Thumbnail',
    url: 'https://example.com/news/protocol-001',
    thumbnail_url: '//via.placeholder.com/150',
    thumbnail_data: null,
    description: 'News with protocol-relative URL (//)',
    fetched_at: now,
    is_seen: false,
    is_saved: false,
    session_date: today,
  },
  {
    id: 'test-video-404-001',
    source: 'test-video-platform',
    category: 'video',
    title: 'Test Video: Failing Thumbnail URL (404)',
    url: 'https://example.com/video/404-001',
    thumbnail_url: 'https://example.com/nonexistent.jpg',
    thumbnail_data: null,
    description: 'Video with invalid thumbnail that will 404',
    fetched_at: now,
    is_seen: false,
    is_saved: false,
    session_date: today,
  },
  {
    id: 'test-gossip-no-thumb-001',
    source: 'test-gossip-feed',
    category: 'gossip',
    title: 'Test Gossip: No Thumbnail Available',
    url: 'https://example.com/gossip/no-thumb-001',
    thumbnail_url: null,
    thumbnail_data: null,
    description: 'Gossip item with both thumbnail fields null',
    fetched_at: now,
    is_seen: false,
    is_saved: false,
    session_date: today,
  },
];

export function getTestStoreState() {
  return {
    items: TEST_ITEMS,
    isDoneWorking: true,
    view: 'detail' as const,
    isLoading: false,
    activeCategory: 'all' as const,
  };
}

export function getTestStoreStateWithUrls() {
  return {
    items: [...TEST_ITEMS, ...TEST_ITEMS_WITH_URLS],
    isDoneWorking: true,
    view: 'detail' as const,
    isLoading: false,
    activeCategory: 'all' as const,
  };
}
