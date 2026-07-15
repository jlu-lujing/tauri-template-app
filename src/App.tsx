import { useState, useRef, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './App.css';
import { useThemeStore } from './stores/themeStore';
import { toggleLang } from './stores/langStore';
import { useTranslation } from 'react-i18next';
import {
  Home,
  Settings,
  LayoutDashboard,
  FolderOpen,
  Sun,
  Moon,
  Languages,
  Menu,
} from 'lucide-react';

/* ── Logo icon (inline SVG — generic hex shape for template) ─ */

function LogoIcon({ size = 16, strokeWidth = 2 }: { size?: number; strokeWidth?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={strokeWidth}>
      <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
      <polyline points="3.27 6.96 12 12.01 20.73 6.96" />
      <line x1="12" y1="22.08" x2="12" y2="12" />
    </svg>
  );
}

/* ── NavButton (sidebar items) ─────────────────────────────── */

interface NavButtonProps {
  icon: React.ComponentType<{ size?: number; strokeWidth?: number }>;
  label: string;
  active?: boolean;
  onClick?: () => void;
  collapsed?: boolean;
}

function NavButton({ icon: Icon, label, active, onClick, collapsed }: NavButtonProps) {
  const [hover, setHover] = useState(false);

  return (
    <button
      onClick={onClick}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      title={label}
      style={{
        display: 'flex',
        alignItems: collapsed ? 'center' : 'center',
        justifyContent: collapsed ? 'center' : 'flex-start',
        gap: 8,
        padding: collapsed ? '0' : '0 10px',
         width: collapsed ? 36 : '100%',
        height: 36,
        borderRadius: 'var(--radius-sm)',
        fontSize: 13,
        fontWeight: 400,
        background: active || hover ? 'var(--sidebar-accent)' : 'transparent',
        color: 'var(--sidebar-foreground)',
        cursor: 'pointer',
        transition: 'background 150ms ease, color 150ms ease',
        border: 'none',
        textAlign: 'left',
      }}
    >
      <Icon size={16} strokeWidth={2} />
      {!collapsed && <span>{label}</span>}
    </button>
  );
}

/* ── Sidebar Footer Tooltip Button ─────────────────────────────── */

interface TooltipButtonProps {
  label: string;
  onClick?: () => void;
  children: React.ReactNode;
  active?: boolean;
}

function TooltipButton({ label, onClick, children, active }: TooltipButtonProps) {
  const [hover, setHover] = useState(false);

  return (
    <button
      title={label}
      onClick={onClick}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        width: 32,
        height: 32,
        borderRadius: 'var(--radius-sm)',
        background: active || hover ? 'var(--sidebar-accent)' : 'transparent',
        color: 'var(--sidebar-foreground)',
        cursor: 'pointer',
        border: 'none',
        transition: 'background 150ms ease',
      }}
    >
      {children}
    </button>
  );
}

/* ── Sidebar tab definitions ─────── */

type TabKey = 'home' | 'dashboard' | 'projects' | 'settings';

const allTabs: Array<{ key: TabKey; icon: React.ComponentType<{ size?: number; strokeWidth?: number }> }> = [
  { key: 'home', icon: Home },
  { key: 'dashboard', icon: LayoutDashboard },
  { key: 'projects', icon: FolderOpen },
];

// Nav sidebar only shows non-settings tabs; settings is accessible via footer button
const navTabs = allTabs.filter((tab) => tab.key !== 'settings');

/* ── App Component ──────────────────────────────────────────────── */

function App() {
  const [collapsed, setCollapsed] = useState(false);
  const [sidebarWidthPx, setSidebarWidthPx] = useState(200);
  const [activeTab, setActiveTab] = useState<TabKey>('home');
  const sidebarRef = useRef<HTMLDivElement>(null);

  const { mode, toggle: toggleTheme } = useThemeStore();
  const t = useTranslation().t;

  /* Persist collapsed/width in localStorage */
  useEffect(() => {
    const saved = localStorage.getItem('sidebar-collapsed');
    if (saved !== null) setCollapsed(JSON.parse(saved));
    const savedWidth = localStorage.getItem('sidebar-width');
    if (savedWidth) setSidebarWidthPx(Number(savedWidth));
  }, []);

  useEffect(() => {
    localStorage.setItem('sidebar-collapsed', JSON.stringify(collapsed));
  }, [collapsed]);

  useEffect(() => {
    localStorage.setItem('sidebar-width', String(sidebarWidthPx));
  }, [sidebarWidthPx]);

  /* Sync collapsed state from other tabs */
  useEffect(() => {
    const handler = (e: StorageEvent) => {
      if (e.key === 'sidebar-collapsed' && e.newValue !== null) {
        setCollapsed(JSON.parse(e.newValue));
      }
      if (e.key === 'sidebar-width' && e.newValue !== null) {
        setSidebarWidthPx(Number(e.newValue));
      }
    };
    window.addEventListener('storage', handler);
    return () => window.removeEventListener('storage', handler);
  }, []);

  const toggleSidebarCollapse = () => setCollapsed((v) => !v);

  const handleMouseDown = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    const startX = e.clientX;
    const startWidth = sidebarWidthPx;
    const minW = collapsed ? 52 : 160;
    const maxW = 600;

    function onMove(ev: MouseEvent) {
      const diff = ev.clientX - startX;
      let newW = Math.max(minW, Math.min(maxW, startWidth + diff));
      setSidebarWidthPx(newW);
    }

    function onUp() {
      document.removeEventListener('mousemove', onMove);
      document.removeEventListener('mouseup', onUp);
      localStorage.setItem('sidebar-width', String(sidebarWidthPx));
    }

    document.addEventListener('mousemove', onMove, { once: false });
    document.addEventListener('mouseup', onUp, { once: true });
  };

  /* ── Render ─────────────────────────────────────────────────── */

  return (
    <div
      className="flex h-screen"
      data-slot="root"
      style={{
        background: 'transparent',
        borderRadius: 12,
        overflow: 'hidden',
        position: 'relative',
      }}
    >
      {/* Drag bar — top area for dragging and double-click to maximize */}
      <div
        style={{
          position: 'absolute',
          top: 0,
          left: 0,
          right: 0,
          height: 46,
          zIndex: 1000,
          userSelect: 'none',
          WebkitUserSelect: 'none',
        }}
        onMouseDown={() => {
          invoke('win_start_drag').catch(() => {});
        }}
        onDoubleClick={() => {
          invoke('win_maximize').catch(() => {});
        }}
      />

      {/* Sidebar */}
       <aside
        id="tauri-sidebar"
        data-slot="sidebar"
        style={{
          width: `${collapsed ? 52 : sidebarWidthPx}px`,
          display: 'flex',
          flexDirection: 'column',
          flexShrink: 0,
          background: 'var(--sidebar)',
          color: 'var(--sidebar-foreground)',
          position: 'relative',
          minWidth: collapsed ? 52 : 160,
          height: '100%',
        }}
      >
        {/* Resize handle — right edge drag area (hidden when collapsed) */}
         {!collapsed && (
          <div
            ref={sidebarRef}
            style={{
              position: 'absolute',
              top: 0,
              right: -3,
              width: 6,
              height: '100%',
              cursor: 'col-resize',
              zIndex: 50,
              userSelect: 'none',
              WebkitUserSelect: 'none',
            }}
            onMouseDown={handleMouseDown}
          />
        )}

        {/* Spacer (matches drag bar height) */}
        <div style={{ height: 46, flexShrink: 0 }} />

        {/* Logo + Title row */}
         <div style={{ padding: '0 8px 8px', display: 'flex', flexDirection: 'column', gap: 4 }}>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: collapsed ? 'center' : 'flex-start', gap: 8, padding: '0 10px', height: 36, borderRadius: 'var(--radius-sm)', width: '100%' }}>
            <LogoIcon size={16} strokeWidth={2} />
            {!collapsed && <span style={{ fontSize: 15, fontWeight: 700, color: 'var(--sidebar-foreground)', letterSpacing: '-0.02em' }}>Tauri App</span>}
          </div>

          {/* Tab buttons */}
          {navTabs.map((tab) => (
            <NavButton
              key={tab.key}
              icon={tab.icon}
              label={t(`nav.${tab.key}`)}
              active={activeTab === tab.key}
              onClick={() => setActiveTab(tab.key)}
              collapsed={collapsed}
            />
          ))}
        </div>

        {/* Footer: pinned to bottom of sidebar */}
        <div style={{
          marginTop: 'auto',
          padding: '0 10px 8px',
          display: 'flex',
          flexDirection: collapsed ? 'column' : 'row',
          alignItems: collapsed ? 'center' : 'center',
          gap: collapsed ? 4 : 4,
        }}>
          {collapsed ? (
            <>
              <TooltipButton key="lang" label={t('settings.languageLabel')} onClick={toggleLang}>
                <Languages size={16} strokeWidth={2} />
              </TooltipButton>
              <TooltipButton key="theme" label={mode === 'light' ? t('settings.themeButtonSwitchToDark') : t('settings.themeButtonSwitchToLight')} onClick={toggleTheme}>
                {mode === 'light' ? <Moon size={16} strokeWidth={2} /> : <Sun size={16} strokeWidth={2} />}
              </TooltipButton>
              <TooltipButton key="settings" label={t('nav.settings')} onClick={() => setActiveTab('settings')} active={activeTab === 'settings'}>
                <Settings size={16} strokeWidth={2} />
              </TooltipButton>
              <TooltipButton key="collapse" label="Open sidebar" onClick={toggleSidebarCollapse}>
                <Menu size={16} strokeWidth={2} />
              </TooltipButton>
            </>
          ) : (
            <>
              <TooltipButton key="collapse" label="Open sidebar" onClick={toggleSidebarCollapse}>
                <Menu size={16} strokeWidth={2} />
              </TooltipButton>
              <TooltipButton key="settings" label={t('nav.settings')} onClick={() => setActiveTab('settings')} active={activeTab === 'settings'}>
                <Settings size={16} strokeWidth={2} />
              </TooltipButton>
              <TooltipButton key="theme" label={mode === 'light' ? t('settings.themeButtonSwitchToDark') : t('settings.themeButtonSwitchToLight')} onClick={toggleTheme}>
                {mode === 'light' ? <Moon size={16} strokeWidth={2} /> : <Sun size={16} strokeWidth={2} />}
              </TooltipButton>
              <TooltipButton key="lang" label={t('settings.languageLabel')} onClick={toggleLang}>
                <Languages size={16} strokeWidth={2} />
              </TooltipButton>
            </>
          )}
        </div>
      </aside>

      {/* Main Content Area */}
      <main data-slot="content" style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
        <div
          style={{
            flex: 1,
            overflow: 'auto',
            padding: '16px 24px',
            background: 'var(--background)',
            borderRadius: 'var(--radius-xl)',
            margin: '8px 8px 8px 0',
            boxShadow: '-1px 0 3px -1px rgba(0,0,0,0.08)',
            minHeight: 0,
          }}
        >
          {activeTab === 'home' && <HomePage />}
          {activeTab === 'dashboard' && <EmptyPage title={t('nav.dashboard')} />}
          {activeTab === 'projects' && <EmptyPage title={t('nav.projects')} />}
          {activeTab === 'settings' && <SettingsPage />}
        </div>
      </main>
    </div>
  );
}

/* ── Empty Page (placeholder) ─────────────────────────────── */

function EmptyPage({ title }: { title: string }) {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', height: '100%', gap: 12 }}>
      <h2 style={{ fontSize: 18, fontWeight: 600, color: 'var(--foreground)', margin: 0 }}>{title}</h2>
    </div>
  );
}

/* ── Home Panel ─────────────────────────────────────── */

function HomePage() {
  const { t } = useTranslation();

  return (
    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', height: '100%', gap: 12 }}>
      <h2 style={{ fontSize: 18, fontWeight: 600, color: 'var(--foreground)', margin: 0 }}>{t('home.welcome')}</h2>
      <p style={{ fontSize: 14, color: 'var(--muted-foreground)', margin: 0 }}>{t('home.description')}</p>
    </div>
  );
}

/* ── Settings Page ─────────────────────────────────────── */

function SettingsPage() {
  const { mode, toggle: toggleTheme } = useThemeStore();
  const { i18n } = useTranslation();
  const t = useTranslation().t;
  const lang = i18n.language as 'zh' | 'en';

  return (
    <div>
      <h1 style={{ fontSize: 16, fontWeight: 700, color: 'var(--foreground)', margin: 0, marginBottom: 12 }}>
        {t('settings.title')}
      </h1>

      <div
        data-slot="card"
        style={{
          padding: 16,
          background: 'var(--card)',
          borderRadius: 'var(--radius)',
          border: '1px solid var(--border)',
        }}
      >
        <h2 style={{ fontSize: 14, fontWeight: 600, margin: 0, marginBottom: 12, color: 'var(--card-foreground)' }}>
          {t('settings.appearance')}
        </h2>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 12 }}>
          <span style={{ fontSize: 13, color: 'var(--muted-foreground)' }}>{t('settings.themeLabel')}</span>
          <button
            onClick={toggleTheme}
            onMouseEnter={(e) => { e.currentTarget.style.background = 'var(--sidebar-accent)'; }}
            onMouseLeave={(e) => { e.currentTarget.style.background = 'transparent'; }}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 6,
              padding: '4px 12px',
              height: 32,
              borderRadius: 'var(--radius-sm)',
              fontSize: 13,
              fontWeight: 500,
              background: 'transparent',
              color: 'var(--sidebar-foreground)',
              border: 'none',
              cursor: 'pointer',
              transition: 'background 150ms ease',
            }}
          >
            {mode === 'light' ? <Moon size={14} /> : <Sun size={14} />}
            {t(`settings.themeButtonSwitchTo${mode === 'light' ? 'Dark' : 'Light'}`)}
          </button>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <span style={{ fontSize: 13, color: 'var(--muted-foreground)' }}>{t('settings.languageLabel')}</span>
         <button
            onClick={() => i18n.changeLanguage(lang === 'zh' ? 'en' : 'zh')}
            onMouseEnter={(e) => { e.currentTarget.style.background = 'var(--sidebar-accent)'; }}
            onMouseLeave={(e) => { e.currentTarget.style.background = 'transparent'; }}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 6,
              padding: '4px 12px',
              height: 32,
              borderRadius: 'var(--radius-sm)',
              fontSize: 13,
              fontWeight: 500,
              background: 'transparent',
              color: 'var(--sidebar-foreground)',
              border: 'none',
              cursor: 'pointer',
              transition: 'background 150ms ease',
            }}
          >
            <Languages size={14} />
            {t(`lang.${lang}`)}
          </button>
        </div>
      </div>
    </div>
  );
}

export default App;
