<script lang="ts">
    import { onMount } from 'svelte';
    import { Marked } from 'marked';
    
    // Import raw markdown files using Vite's ?raw modifier
    import readmeRaw from '../../../docs/README.md?raw';
    import architectureRaw from '../../../docs/architecture.md?raw';
    import featuresRaw from '../../../docs/features.md?raw';
    import installationRaw from '../../../docs/installation.md?raw';
    import developmentRaw from '../../../docs/development.md?raw';

    const marked = new Marked();

    const docsMap: Record<string, { title: string; content: string }> = {
        introduction: { title: "Introduction", content: readmeRaw },
        architecture: { title: "System Architecture", content: architectureRaw },
        features: { title: "Features & Ingestion", content: featuresRaw },
        installation: { title: "Installation & Setup", content: installationRaw },
        development: { title: "Developer Guide", content: developmentRaw }
    };

    let activeDocId = $state("introduction");
    let renderedHtml = $derived.by(() => {
        const raw = docsMap[activeDocId]?.content || "";
        // Replace relative and markdown file links so navigation works smoothly in the web reader
        const processed = raw.replace(/\b(docs\/|file:\/\/[^)\s]*\/)?([a-zA-Z0-9_-]+)\.md\b/g, (match, prefix, filename) => {
            if (filename === 'README') return '#docs-introduction';
            return `#docs-${filename}`;
        });
        return marked.parse(processed) as string;
    });

    function selectDoc(id: string) {
        activeDocId = id;
        const panel = document.querySelector('.doc-content-panel');
        if (panel) {
            panel.scrollTop = 0;
        }
    }

    function handleHashChange() {
        const hash = window.location.hash;
        if (hash.startsWith('#docs-')) {
            const docId = hash.replace('#docs-', '');
            if (docsMap[docId]) {
                activeDocId = docId;
            } else if (docId === 'introduction' || docId === 'README') {
                activeDocId = 'introduction';
            }
        }
    }

    onMount(() => {
        window.addEventListener('hashchange', handleHashChange);
        handleHashChange();
        return () => {
            window.removeEventListener('hashchange', handleHashChange);
        };
    });
</script>

<div class="docs-container">
    <!-- Sidebar -->
    <aside class="docs-sidebar">
        <div class="sidebar-header">
            <h4>Documentation</h4>
        </div>
        <nav class="sidebar-nav">
            {#each Object.entries(docsMap) as [id, doc]}
                <button 
                    class="nav-item" 
                    class:active={activeDocId === id}
                    onclick={() => selectDoc(id)}
                >
                    <span class="indicator"></span>
                    <span class="title">{doc.title}</span>
                </button>
            {/each}
        </nav>
        <div class="sidebar-footer">
            <a href="https://github.com/n123xyz/Garmin-Goblin/tree/main/docs" target="_blank" rel="noopener noreferrer" class="view-raw-link">
                <span>View Raw Docs on GitHub</span>
                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path><polyline points="15 3 21 3 21 9"></polyline><line x1="10" y1="14" x2="21" y2="3"></line></svg>
            </a>
        </div>
    </aside>

    <!-- Main Content Panel -->
    <main class="doc-content-panel">
        <div class="markdown-body">
            <!-- eslint-disable-next-line svelte/no-at-html-tags -->
            {@html renderedHtml}
        </div>
    </main>
</div>

<style>
    .docs-container {
        display: flex;
        flex: 1;
        max-width: 1300px;
        margin: 0 auto;
        width: 100%;
        min-height: calc(100vh - 140px);
        padding: 2rem 1.5rem;
        gap: 2.5rem;
    }

    .docs-sidebar {
        width: 280px;
        flex-shrink: 0;
        background: rgba(19, 23, 21, 0.6);
        border: 1px solid var(--border);
        border-radius: 16px;
        padding: 1.5rem;
        display: flex;
        flex-direction: column;
        height: fit-content;
        position: sticky;
        top: 90px;
        backdrop-filter: blur(12px);
    }

    .sidebar-header h4 {
        margin: 0 0 1.25rem 0;
        font-size: 0.9rem;
        text-transform: uppercase;
        letter-spacing: 0.08em;
        color: var(--accent-light);
        font-weight: 750;
    }

    .sidebar-nav {
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
    }

    .nav-item {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        padding: 0.65rem 0.85rem;
        background: transparent;
        border: 1px solid transparent;
        border-radius: 8px;
        color: var(--text-muted);
        font-size: 0.95rem;
        font-weight: 550;
        cursor: pointer;
        text-align: left;
        transition: all 0.15s ease;
        width: 100%;
    }

    .nav-item:hover {
        background: rgba(74, 222, 128, 0.08);
        color: var(--text-heading);
    }

    .nav-item.active {
        background: rgba(74, 222, 128, 0.15);
        border-color: rgba(74, 222, 128, 0.35);
        color: var(--accent-light);
    }

    .indicator {
        width: 6px;
        height: 6px;
        border-radius: 50%;
        background: transparent;
        transition: background 0.15s ease;
    }

    .nav-item.active .indicator {
        background: var(--accent-light);
        box-shadow: 0 0 8px var(--accent);
    }

    .sidebar-footer {
        margin-top: 2rem;
        padding-top: 1rem;
        border-top: 1px solid var(--border);
    }

    .view-raw-link {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        color: var(--text-muted);
        font-size: 0.85rem;
        text-decoration: none;
        transition: color 0.15s ease;
    }

    .view-raw-link:hover {
        color: var(--accent-light);
    }

    .doc-content-panel {
        flex: 1;
        min-width: 0;
        background: rgba(19, 23, 21, 0.4);
        border: 1px solid var(--border);
        border-radius: 16px;
        padding: 2.5rem 3rem;
        backdrop-filter: blur(12px);
    }

    @media (max-width: 850px) {
        .docs-container {
            flex-direction: column;
            padding: 1rem;
        }

        .docs-sidebar {
            width: 100%;
            position: relative;
            top: 0;
        }

        .doc-content-panel {
            padding: 1.5rem;
        }
    }
</style>
