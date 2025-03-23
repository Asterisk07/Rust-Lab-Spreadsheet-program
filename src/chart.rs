use plotters::prelude::*;
use std::f64::consts::PI;

// use plotters::prelude::*;
// use std::f64::consts::PI;

fn draw_pie_chart(file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(file_name, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let center = (400, 300);
    let radius = 150;

    // Data: percentages
    let data = vec![("A", 35.0), ("B", 25.0), ("C", 20.0), ("D", 20.0)];
    let total: f64 = data.iter().map(|(_, v)| *v).sum();

    let mut start_angle = 0.0;

    for (i, (label, value)) in data.iter().enumerate() {
        let angle = 2.0 * PI * (*value / total);
        let end_angle = start_angle + angle;

        let color = Palette99::pick(i).mix(0.8);

        // Draw the pie slice
        root.draw(&PathElement::new(
            vec![
                center,
                (
                    center.0 + (radius as f64 * start_angle.cos()) as i32,
                    center.1 - (radius as f64 * start_angle.sin()) as i32,
                ),
                (
                    center.0 + (radius as f64 * end_angle.cos()) as i32,
                    center.1 - (radius as f64 * end_angle.sin()) as i32,
                ),
            ],
            ShapeStyle {
                color: color.to_rgba(),
                filled: true,
                stroke_width: 1,
            },
        ))?;

        // Add text labels
        let label_x = center.0 + (radius as f64 * (start_angle + angle / 2.0).cos() * 0.7) as i32;
        let label_y = center.1 - (radius as f64 * (start_angle + angle / 2.0).sin() * 0.7) as i32;

        root.draw(&Text::new(
            format!("{}: {:.1}%", label, value),
            (label_x, label_y),
            ("sans-serif", 20.0).into_font().color(&BLACK),
        ))?;

        start_angle = end_angle;
    }

    println!("Pie chart saved to {}", file_name);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Line Chart
    let root = BitMapBackend::new("line_chart.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Line Chart", ("sans-serif", 40))
        .build_cartesian_2d(0f32..10f32, 0f32..100f32)?;

    chart.configure_mesh().draw()?;
    chart.draw_series(LineSeries::new(
        (0..10).map(|x| (x as f32, (x * x) as f32)),
        &BLUE,
    ))?;
    println!("Line chart saved!");

    // Bar Chart
    let root = BitMapBackend::new("bar_chart.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Bar Chart", ("sans-serif", 40))
        .build_cartesian_2d(0..5, 0..100)?;

    chart.configure_mesh().draw()?;
    chart.draw_series((0..5).map(|x| {
        let y = (x * x) + 10;
        Rectangle::new([(x, 0), (x + 1, y)], BLUE.filled())
    }))?;
    println!("Bar chart saved!");

    // Pie Chart
    draw_pie_chart("pie_chart.png")?;

    // Scatter Plot
    let root = BitMapBackend::new("scatter_plot.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Scatter Plot", ("sans-serif", 40))
        .build_cartesian_2d(0f32..10f32, 0f32..100f32)?;

    chart.configure_mesh().draw()?;
    chart.draw_series((0..10).map(|x| Circle::new((x as f32, (x * x) as f32), 5, RED.filled())))?;
    println!("Scatter plot saved!");

    // Histogram
    let root = BitMapBackend::new("histogram.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Histogram", ("sans-serif", 40))
        .build_cartesian_2d(0..10, 0..10)?;

    chart.configure_mesh().draw()?;
    chart.draw_series((0..10).map(|x| Rectangle::new([(x, 0), (x + 1, x % 5)], GREEN.filled())))?;
    println!("Histogram saved!");

    // Combined Chart
    let root = BitMapBackend::new("combined_chart.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Combined Chart", ("sans-serif", 40))
        .build_cartesian_2d(0f32..10f32, 0f32..100f32)?;

    chart.configure_mesh().draw()?;

    // Line series
    chart.draw_series(LineSeries::new(
        (0..10).map(|x| (x as f32, (x * x) as f32)),
        &BLUE,
    ))?;

    // Scatter series
    chart
        .draw_series((0..10).map(|x| Circle::new((x as f32, (x * 10) as f32), 5, RED.filled())))?;

    println!("Combined chart saved!");

    Ok(())
}
