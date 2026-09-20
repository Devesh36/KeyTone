export type Platform = "macos" | "windows" | "linux";
export interface ReleaseAsset { name: string; browser_download_url: string; size: number }
export interface ReleaseResponse { tag_name: string; assets: ReleaseAsset[] }

export function detectPlatform(userAgent: string, platform: string, touchPoints = 0): Platform | null {
  if (/android|iphone|ipad|ipod/i.test(userAgent) || (/mac/i.test(platform) && touchPoints > 1)) return null;
  if (/win/i.test(platform)) return "windows";
  if (/mac/i.test(platform)) return "macos";
  if (/linux|x11/i.test(`${platform} ${userAgent}`)) return "linux";
  return null;
}

export function pickAsset(platform: Platform, assets: ReleaseAsset[]): ReleaseAsset | undefined {
  const extensions: Record<Platform, string[]> = {
    macos: [".dmg"], windows: [".msi", ".exe"], linux: [".appimage", ".deb"],
  };
  return assets
    .filter((asset) => extensions[platform].some((extension) => asset.name.toLowerCase().endsWith(extension)))
    .map((asset) => {
      const name = asset.name.toLowerCase();
      const preference = extensions[platform].findIndex((extension) => name.endsWith(extension));
      return { asset, score: (2 - preference) * 10 + (name.includes("universal") ? 4 : 0) + (/x64|amd64/.test(name) ? 1 : 0) };
    })
    .sort((a, b) => b.score - a.score)[0]?.asset;
}

export function isRelease(value: unknown): value is ReleaseResponse {
  if (typeof value !== "object" || value === null) return false;
  const release = value as Partial<ReleaseResponse>;
  return typeof release.tag_name === "string" && Array.isArray(release.assets) && release.assets.every((asset: unknown) => {
    if (typeof asset !== "object" || asset === null) return false;
    const item = asset as Partial<ReleaseAsset>;
    return typeof item.name === "string" && typeof item.size === "number" && item.size > 0
      && typeof item.browser_download_url === "string"
      && item.browser_download_url.startsWith("https://github.com/Devesh36/KeyTone/releases/download/");
  });
}
