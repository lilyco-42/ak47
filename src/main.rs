// 隐藏控制台（纯 Windows GUI 应用）
#![windows_subsystem = "windows"]

use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::rc::Rc;

use winsafe::{self as w, co, gui, prelude::*};

// ═══════════════════════════════════════════════════════════════
// ---------- 核心功能函数 ----------

fn read_file_content(path: &str) -> Result<String, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    Ok(content.replace("\r\n", "\n"))
}

fn count_chinese_words(text: &str) -> HashMap<String, usize> {
    use jieba_rs::Jieba;
    let jieba = Jieba::new();
    let mut map = HashMap::new();
    let tokens = jieba.cut(text, false);
    for token in tokens {
        let word = &token.word;
        if word.chars().all(|c| c.is_alphabetic()) && word.len() > 1 {
            *map.entry(word.to_string()).or_insert(0) += 1;
        }
    }
    map
}

fn generate_wordcloud_html(
    word_count: &HashMap<String, usize>,
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    use serde_json::json;

    let mut entries: Vec<_> = word_count.iter().collect();
    entries.sort_by(|a, b| b.1.cmp(a.1));

    let top_n = 50;
    let top_entries = &entries[..entries.len().min(top_n)];

    let cloud_data: Vec<_> = top_entries
        .iter()
        .map(|(word, count)| json!({"name": word, "value": count}))
        .collect();

    let bar_top_n = 20;
    let bar_entries = &entries[..entries.len().min(bar_top_n)];
    let words: Vec<_> = bar_entries.iter().map(|(w, _)| w.as_str()).collect();
    let counts: Vec<_> = bar_entries.iter().map(|(_, c)| *c).collect();

    let cloud_json = serde_json::to_string(&cloud_data)?;
    let words_json = serde_json::to_string(&words)?;
    let counts_json = serde_json::to_string(&counts)?;

    let html_content = format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>词频可视化</title>
    <script src="https://cdn.jsdelivr.net/npm/echarts@5/dist/echarts.min.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/echarts-wordcloud@2/dist/echarts-wordcloud.min.js"></script>
    <style>
        body {{ font-family: 'Microsoft YaHei', sans-serif; background: #f0f2f5; margin: 0; padding: 20px; }}
        .header {{ text-align: center; margin-bottom: 20px; }}
        .header h1 {{ color: #1a1a2e; font-size: 28px; margin: 0; }}
        .header p {{ color: #666; font-size: 14px; margin: 4px 0 0 0; }}
        .container {{ display: flex; flex-wrap: wrap; justify-content: center; gap: 24px; max-width: 1300px; margin: 0 auto; }}
        .chart-box {{
            background: white;
            border-radius: 12px;
            box-shadow: 0 4px 16px rgba(0,0,0,0.08);
            padding: 24px;
            flex: 1 1 560px;
            max-width: 640px;
            min-width: 400px;
        }}
        .chart-box h2 {{
            text-align: center;
            margin: 0 0 16px 0;
            color: #333;
            font-size: 18px;
            border-bottom: 2px solid #5470c6;
            padding-bottom: 8px;
        }}
        .chart {{ width: 100%; height: 400px; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>📊 歌词词频分析</h1>
        <p>基于 ECharts 的词频可视化</p>
    </div>
    <div class="container">
        <div class="chart-box">
            <h2>词云</h2>
            <div id="wordcloud" class="chart"></div>
        </div>
        <div class="chart-box">
            <h2>高频词柱状图</h2>
            <div id="bar" class="chart"></div>
        </div>
    </div>
    <script>
        const cloudData = {cloud_json};
        const cloudChart = echarts.init(document.getElementById('wordcloud'));
        cloudChart.setOption({{
            series: [{{
                type: 'wordCloud',
                gridSize: 8,
                sizeRange: [14, 65],
                rotationRange: [-90, 90],
                rotationStep: 45,
                shape: 'circle',
                width: '90%',
                height: '90%',
                left: 'center',
                top: 'center',
                textStyle: {{
                    fontFamily: 'Microsoft YaHei, sans-serif',
                    fontWeight: 'bold',
                    color: function () {{
                        return 'rgb(' + [
                            Math.round(Math.random() * 120 + 60),
                            Math.round(Math.random() * 120 + 60),
                            Math.round(Math.random() * 120 + 60)
                        ].join(',') + ')';
                    }}
                }},
                data: cloudData.map(function(item) {{
                    return {{ name: item.name, value: item.value }};
                }})
            }}]
        }});

        const barChart = echarts.init(document.getElementById('bar'));
        barChart.setOption({{
            tooltip: {{
                trigger: 'axis',
                axisPointer: {{ type: 'shadow' }}
            }},
            grid: {{
                left: '8%',
                right: '8%',
                top: '5%',
                bottom: '15%'
            }},
            xAxis: {{
                type: 'category',
                data: {words_json},
                axisLabel: {{
                    interval: 0,
                    rotate: 0,
                    fontSize: 11,
                    formatter: function (value) {{
                        return value.length > 4
                            ? value.substring(0, 4) + '\\n' + value.substring(4)
                            : value;
                    }}
                }}
            }},
            yAxis: {{
                type: 'value',
                name: '出现次数',
                nameTextStyle: {{ fontSize: 12 }}
            }},
            series: [{{
                name: '词频',
                type: 'bar',
                data: {counts_json},
                barWidth: '50%',
                itemStyle: {{
                    color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
                        {{ offset: 0, color: '#667eea' }},
                        {{ offset: 1, color: '#5470c6' }}
                    ]),
                    borderRadius: [4, 4, 0, 0]
                }}
            }}]
        }});

        window.addEventListener('resize', function() {{
            cloudChart.resize();
            barChart.resize();
        }});
    </script>
</body>
</html>
"#,
        cloud_json = cloud_json,
        words_json = words_json,
        counts_json = counts_json,
    );

    fs::write(output_path, html_content)?;

    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", output_path])
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(output_path).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(output_path)
            .spawn();
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════
// ---------- WinSafe GUI (0.0.27) ----------

#[derive(Clone)]
struct AppState {
    current_path: String,
    word_count: HashMap<String, usize>,
}

struct MainWindow {
    wnd: gui::WindowMain,
    path_label: gui::Label,
    open_btn: gui::Button,
    generate_btn: gui::Button,
    clear_btn: gui::Button,
    quit_btn: gui::Button,
    list_view: gui::ListView,
    status_bar: gui::StatusBar,
    state: Rc<RefCell<AppState>>,
}

impl MainWindow {
    fn new() -> w::AnyResult<Self> {
        let win_w = 780i32;
        let win_h = 540i32;

        let wnd = gui::WindowMain::new(gui::WindowMainOpts {
            title: "词频分析器 — AK47",
            size: gui::dpi(win_w, win_h),
            style: gui::WindowMainOpts::default().style
                | co::WS::MINIMIZEBOX
                | co::WS::MAXIMIZEBOX
                | co::WS::SIZEBOX,
            ..Default::default()
        });

        // ── 文件标签 ──
        let _file_label = gui::Label::new(
            &wnd,
            gui::LabelOpts {
                text: "📂 选择文件：",
                position: gui::dpi(14, 14),
                size: gui::dpi(110, 24),
                ..Default::default()
            },
        );

        let path_label = gui::Label::new(
            &wnd,
            gui::LabelOpts {
                text: "（未选择文件）",
                position: gui::dpi(124, 14),
                size: gui::dpi(530, 24),
                ..Default::default()
            },
        );

        // ── 按钮 ──
        let open_btn = gui::Button::new(
            &wnd,
            gui::ButtonOpts {
                text: "📁 打开文件",
                position: gui::dpi(14, 46),
                width: gui::dpi_x(110),
                height: gui::dpi_y(30),
                ..Default::default()
            },
        );

        let generate_btn = gui::Button::new(
            &wnd,
            gui::ButtonOpts {
                text: "📊 生成词云",
                position: gui::dpi(132, 46),
                width: gui::dpi_x(110),
                height: gui::dpi_y(30),
                ..Default::default()
            },
        );

        let clear_btn = gui::Button::new(
            &wnd,
            gui::ButtonOpts {
                text: "🗑 清除",
                position: gui::dpi(250, 46),
                width: gui::dpi_x(80),
                height: gui::dpi_y(30),
                ..Default::default()
            },
        );

        let quit_btn = gui::Button::new(
            &wnd,
            gui::ButtonOpts {
                text: "❌ 退出",
                position: gui::dpi(win_w - 96, 46),
                width: gui::dpi_x(80),
                height: gui::dpi_y(30),
                ..Default::default()
            },
        );

        // ── 列表视图 ──
        let list_view = gui::ListView::new(
            &wnd,
            gui::ListViewOpts {
                position: gui::dpi(14, 86),
                size: gui::dpi(win_w - 28, win_h - 160),
                columns: &[("序号", 55), ("词语", 300), ("词频", 80)],
                resize_behavior: (gui::Horz::Resize, gui::Vert::Resize),
                ..Default::default()
            },
        );

        // ── 状态栏 ──
        let status_bar = gui::StatusBar::new(
            &wnd,
            &[gui::SbPart::Proportional(3), gui::SbPart::Proportional(2)],
        );

        let state = Rc::new(RefCell::new(AppState {
            current_path: String::new(),
            word_count: HashMap::new(),
        }));

        let this = Self {
            wnd,
            path_label,
            open_btn,
            generate_btn,
            clear_btn,
            quit_btn,
            list_view,
            status_bar,
            state,
        };

        this.events();
        Ok(this)
    }

    fn events(&self) {
        let wnd = self.wnd.clone();
        let path_label = self.path_label.clone();
        let open_btn = self.open_btn.clone();
        let generate_btn = self.generate_btn.clone();
        let clear_btn = self.clear_btn.clone();
        let quit_btn = self.quit_btn.clone();
        let list_view = self.list_view.clone();
        let status_bar = self.status_bar.clone();
        let state = self.state.clone();

        // ── 窗口创建时初始化 ──
        let status_bar_init = status_bar.clone();
        let generate_btn_init = generate_btn.clone();
        wnd.on().wm_create({
            move |_| -> w::AnyResult<i32> {
                status_bar_init
                    .parts()
                    .set_texts(&[Some("就绪 — 点击「打开文件」选择文本文件"), Some("")]);
                generate_btn_init.hwnd().EnableWindow(false);
                Ok(0)
            }
        });

        // ── 窗口关闭 ──
        wnd.on().wm_close({
            move || -> w::AnyResult<()> {
                w::PostQuitMessage(0);
                Ok(())
            }
        });

        // ── 打开文件 ──
        open_btn.on().bn_clicked({
            let wnd = wnd.clone();
            let state = state.clone();
            let path_label = path_label.clone();
            let list_view = list_view.clone();
            let generate_btn = generate_btn.clone();
            let status_bar = status_bar.clone();
            move || -> w::AnyResult<()> {
                let file_path = match rfd::FileDialog::new()
                    .add_filter("文本文件", &["txt", "md", "lrc"])
                    .add_filter("所有文件", &["*"])
                    .set_title("选择文本文件")
                    .pick_file()
                {
                    Some(path) => path,
                    None => return Ok(()),
                };

                let path_str = file_path.to_string_lossy().to_string();

                match read_file_content(&path_str) {
                    Ok(content) => {
                        // 更新路径标签
                        path_label.hwnd().SetWindowText(&path_str)?;

                        let result = count_chinese_words(&content);
                        let unique_count = result.len();

                        {
                            let mut s = state.borrow_mut();
                            s.current_path = path_str.clone();
                            s.word_count = result.clone();
                        }

                        // 填充 ListView
                        list_view.set_redraw(false);
                        let _ = list_view.items().delete_all();

                        let mut entries: Vec<_> = result.iter().collect();
                        entries.sort_by(|a, b| b.1.cmp(a.1));

                        for (i, (word, count)) in entries.iter().enumerate().take(100) {
                            list_view.items().add(
                                &[&(i + 1).to_string(), word.as_str(), &count.to_string()],
                                None,
                                (),
                            )?;
                        }

                        list_view.set_redraw(true);
                        list_view.hwnd().InvalidateRect(None, true)?;

                        // 启用生成按钮
                        generate_btn.hwnd().EnableWindow(true);

                        // 更新状态栏
                        let sb_left = format!("文件：{}", path_str);
                        let sb_right = format!("共 {} 个不重复词", unique_count);
                        status_bar
                            .parts()
                            .set_texts(&[Some(sb_left.as_str()), Some(sb_right.as_str())]);
                    }
                    Err(e) => {
                        let _ = wnd.hwnd().MessageBox(
                            &format!("读取文件失败：{}", e),
                            "错误",
                            co::MB::OK | co::MB::ICONERROR,
                        );
                    }
                }
                Ok(())
            }
        });

        // ── 生成词云 ──
        generate_btn.on().bn_clicked({
            let state = state.clone();
            let wnd = wnd.clone();
            move || -> w::AnyResult<()> {
                let (path, result) = {
                    let s = state.borrow();
                    (s.current_path.clone(), s.word_count.clone())
                };
                if path.is_empty() {
                    let _ = wnd.hwnd().MessageBox(
                        "请先选择一个文件",
                        "提示",
                        co::MB::OK | co::MB::ICONINFORMATION,
                    );
                    return Ok(());
                }
                if result.is_empty() {
                    let _ = wnd.hwnd().MessageBox(
                        "没有可用的统计结果",
                        "提示",
                        co::MB::OK | co::MB::ICONINFORMATION,
                    );
                    return Ok(());
                }
                let output_path = format!("{}.html", path);
                match generate_wordcloud_html(&result, &output_path) {
                    Ok(()) => {
                        let _ = wnd.hwnd().MessageBox(
                            &format!("词云已生成并打开：\n{}", output_path),
                            "成功",
                            co::MB::OK | co::MB::ICONINFORMATION,
                        );
                    }
                    Err(e) => {
                        let _ = wnd.hwnd().MessageBox(
                            &format!("生成词云失败：{}", e),
                            "错误",
                            co::MB::OK | co::MB::ICONERROR,
                        );
                    }
                }
                Ok(())
            }
        });

        // ── 清除 ──
        clear_btn.on().bn_clicked({
            let list_view = list_view.clone();
            let path_label = path_label.clone();
            let generate_btn = generate_btn.clone();
            let status_bar = status_bar.clone();
            let state = state.clone();
            move || -> w::AnyResult<()> {
                let _ = list_view.items().delete_all();
                {
                    let mut s = state.borrow_mut();
                    s.current_path.clear();
                    s.word_count.clear();
                }
                path_label.hwnd().SetWindowText("（未选择文件）")?;
                generate_btn.hwnd().EnableWindow(false);
                status_bar.parts().set_texts(&[Some("就绪"), Some("")]);
                Ok(())
            }
        });

        // ── 退出 ──
        quit_btn.on().bn_clicked({
            move || -> w::AnyResult<()> {
                w::PostQuitMessage(0);
                Ok(())
            }
        });
    }
}

fn main() {
    w::InitCommonControls();
    if let Err(e) = (|| -> w::AnyResult<()> {
        let main_window = MainWindow::new()?;
        main_window.wnd.run_main(None)?;
        Ok(())
    })() {
        w::HWND::NULL
            .MessageBox(
                &format!("程序出错：{}", e),
                "AK47 错误",
                co::MB::OK | co::MB::ICONERROR,
            )
            .ok();
    }
}
