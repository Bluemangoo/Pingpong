import { DefaultTheme } from "vitepress";

export function nav(): DefaultTheme.NavItem[] {
    return [
        {
            text: "Guide",
            link: "/getting-started/introduction",
            activeMatch: "/guide/"
        },
        {
            text: "Configuration",
            link: "/configuration/config-file",
            activeMatch: "/configuration/"
        }
    ];
}

export function sidebar(): DefaultTheme.SidebarMulti {
    return {
        "/getting-started/": {
            base: "",
            items: [
                sidebarItem.gettingStarted(false),
                sidebarItem.configuration(false)
            ]
        },
        "/configuration/": {
            base: "",
            items: [
                sidebarItem.gettingStarted(true),
                sidebarItem.configuration(false)
            ]
        }
    };
}

const sidebarItem = {
    gettingStarted(collapsed: boolean): DefaultTheme.SidebarItem {
        return {
            text: "Getting Started",
            collapsed,
            items: [
                {
                    text: "Introduction",
                    link: "/getting-started/introduction"
                },
                {
                    text: "Getting Started",
                    link: "/getting-started/getting-started"
                }
            ]
        };
    },
    configuration(collapsed: boolean): DefaultTheme.SidebarItem {
        return {
            text: "Configuration",
            collapsed,
            items: [
                {
                    text: "Command Line Arguments",
                    link: "/configuration/command-line-arguments"
                },
                {
                    text: "Config File",
                    link: "/configuration/config-file"
                },
                {
                    text: "Server",
                    link: "/configuration/server"
                },
                {
                    text: "Source",
                    link: "/configuration/source"
                },
                {
                    text: "Rewrite",
                    link: "/configuration/rewrite"
                },
                {
                    text: "Location",
                    link: "/configuration/location"
                }
            ]
        };
    }
};
