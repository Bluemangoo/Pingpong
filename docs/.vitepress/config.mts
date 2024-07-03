import { defineConfig } from "vitepress";
import { nav as navEn, sidebar as sidebarEn } from "./locale/en";

export default defineConfig({
    rewrites: {
        "index.md": "index.md",
        ":file(.*)/index.md": ":file/index.md",
        ":file(.*).md": ":file/index.md"
    },

    locales: {
        root: {
            label: "English",
            lang: "en",
            title: "Pingpong",
            titleTemplate: "Pingpong Docs",
            description: "A Reverse proxy powered by Pingora",
            themeConfig: {
                editLink: {
                    pattern:
                        "https://github.com/Bluemangoo/Pingpong/edit/master/docs/:path",
                    text: "Edit this page on GitHub"
                },
                nav: navEn(),
                sidebar: sidebarEn()
            },
            head: [["meta", { name: "og:locale", content: "en" }]]
        }
    },

    srcExclude: [],

    lastUpdated: false,
    cleanUrls: true,

    sitemap: {
        hostname: "https://pingpong.bluemangoo.net/"
    },

    /* prettier-ignore */
    head: [
        ["link", { rel: "icon", href: "/favicon.ico" }],
        ["meta", { name: "theme-color", content: "#FF9900" }],
        ["meta", { name: "og:type", content: "website" }],
        ["meta", { name: "og:site_name", content: "Pingpong" }]
        // ['meta', { name: 'og:image', content: '' }],
    ],

    themeConfig: {
        logo: { src: "/favicon.ico", width: 24, height: 24 },

        search: {
            provider: "local"
        },

        socialLinks: [
            { icon: "github", link: "https://github.com/Bluemangoo/Pingpong" }
        ],

        footer: {
            message: "Released under the GPL-3.0 License.",
            copyright: "Copyright © 2023-present Bluemangoo"
        }
    }
});
