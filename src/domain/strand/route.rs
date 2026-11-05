use super::geometry::Rect;
use super::layout::{
    KindSpec, MARGIN, PLATE, PLATE_GAP, ROW, StrandSpec, plan, spans_in, thread_y,
};

const CLEAR: f32 = 10.0;

#[derive(Debug, Clone, Copy)]
pub struct RouteSpec<'a> {
    pub output: [f32; 2],
    pub kinds: &'a [KindSpec],
    pub cancel: f32,
    pub done: f32,
    pub extras: &'a [f32],
    pub boxes: &'a [Rect],
    pub crown: f32,
    pub pad: f32,
    pub gem: f32,
    pub edge: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Route {
    pub lane: f32,
    pub margin: f32,
    pub spans: Vec<[f32; 2]>,
    pub dock: Vec<[f32; 2]>,
}

impl Route {
    pub fn rest(height: f32, spans: Vec<[f32; 2]>) -> Self {
        Self { lane: thread_y(height), margin: MARGIN, spans, dock: Vec::new() }
    }
}

impl<'a> RouteSpec<'a> {
    fn strand(&self, spans: &'a [[f32; 2]], margin: f32, dock: &'a [[f32; 2]]) -> StrandSpec<'a> {
        StrandSpec {
            width: self.output[0],
            kinds: self.kinds,
            cancel: self.cancel,
            done: self.done,
            extras: self.extras,
            open: None,
            obstacles: spans,
            margin,
            dock,
        }
    }

    fn band(&self, lane: f32, top: f32) -> Vec<[f32; 2]> {
        spans_in(self.boxes.iter().copied(), [lane - top, lane + PLATE / 2.0 + ROW])
    }

    fn inline(&self, lane: f32) -> Option<Route> {
        let narrow = self.band(lane, PLATE / 2.0 + ROW);
        let closed = plan(&self.strand(&narrow, MARGIN, &[]));
        let spans = if closed.folded.iter().any(|folded| *folded) {
            self.band(lane, self.crown + ROW)
        } else {
            narrow
        };
        let strand = self.strand(&spans, MARGIN, &[]);
        let fits = (0..self.kinds.len())
            .map(Some)
            .chain([None])
            .all(|open| plan(&StrandSpec { open, ..strand }).fits);
        fits.then(|| Route { lane, margin: MARGIN, spans: spans.clone(), dock: Vec::new() })
    }

    fn docked(&self, lane: f32) -> Option<Route> {
        let dock = self.spot(lane)?;
        let spans = self.band(lane, self.crown + ROW);
        let margin = [MARGIN, self.edge]
            .into_iter()
            .find(|margin| plan(&self.strand(&spans, *margin, &dock)).fits)?;
        Some(Route { lane, margin, spans: spans.clone(), dock })
    }

    fn spot(&self, lane: f32) -> Option<Vec<[f32; 2]>> {
        let widths: Vec<f32> =
            self.extras.iter().copied().chain([self.cancel, self.done]).collect();
        let goal = [self.output[0] - self.edge, lane - self.crown - ROW];
        let band = Rect::new(
            0.0,
            lane - self.crown - ROW,
            self.output[0],
            self.crown + 2.0 * ROW + PLATE / 2.0,
        );
        let mut blocked: Vec<Rect> =
            self.boxes.iter().map(|bounds| bounds.padded(self.pad + self.gem)).collect();
        blocked.push(band);
        let mut best: Option<(f32, Vec<[f32; 2]>)> = None;
        for size in (1..=widths.len()).rev() {
            let lines: Vec<&[f32]> = widths.rchunks(size).rev().collect();
            let wide = lines.iter().map(|line| line_width(line)).fold(0.0, f32::max);
            let tall = lines.len() as f32 * (PLATE + ROW) - ROW;
            for corner in self.corners(lane, [wide, tall]) {
                let block = Rect::new(corner[0], corner[1], wide, tall);
                let free =
                    self.on_screen(block) && !blocked.iter().any(|rect| rect.overlaps(block));
                let cost = (block.right() - goal[0]).hypot(block.bottom() - goal[1]);
                if free && best.as_ref().is_none_or(|(least, _)| cost < *least) {
                    best = Some((cost, flow(&lines, block)));
                }
            }
        }
        best.map(|(_, spots)| spots)
    }

    fn on_screen(&self, block: Rect) -> bool {
        block.x >= self.edge
            && block.y >= self.edge
            && block.right() <= self.output[0] - self.edge
            && block.bottom() <= self.output[1] - self.edge
    }

    fn corners(&self, lane: f32, size: [f32; 2]) -> Vec<[f32; 2]> {
        let [wide, tall] = size;
        let [screen, high] = self.output;
        let mut xs = vec![screen - self.edge - wide, self.edge];
        let mut ys = vec![
            lane - self.crown - ROW - tall,
            lane + PLATE / 2.0 + ROW,
            high - self.edge - tall,
            self.edge,
        ];
        for bounds in self.boxes {
            let near = bounds.padded(self.pad);
            xs.extend([near.x - wide - CLEAR, near.right() + CLEAR]);
            ys.extend([near.y - tall - CLEAR, near.bottom() + CLEAR]);
        }
        ys.iter().flat_map(|y| xs.iter().map(move |x| [*x, *y])).collect()
    }

    fn lanes(&self) -> Vec<f32> {
        let [_, high] = self.output;
        let half = PLATE / 2.0;
        let rest = thread_y(high);
        let mut lanes: Vec<f32> = self
            .boxes
            .iter()
            .map(|bounds| bounds.padded(self.pad))
            .flat_map(|near| [near.y - half - CLEAR, near.bottom() + self.crown + CLEAR])
            .chain([self.edge + self.crown])
            .filter(|lane| *lane - self.crown >= self.edge && *lane + half <= high - self.edge)
            .collect();
        lanes.sort_by(|one, other| (*one - rest).abs().total_cmp(&(*other - rest).abs()));
        lanes.insert(0, rest);
        lanes
    }
}

fn line_width(line: &[f32]) -> f32 {
    line.iter().sum::<f32>() + PLATE_GAP * line.len().saturating_sub(1) as f32
}

fn flow(lines: &[&[f32]], block: Rect) -> Vec<[f32; 2]> {
    let mut spots = Vec::new();
    let mut y = block.y;
    for line in lines {
        let mut x = block.right() - line_width(line);
        for width in *line {
            spots.push([x, y]);
            x += width + PLATE_GAP;
        }
        y += PLATE + ROW;
    }
    spots
}

pub fn route(spec: &RouteSpec<'_>) -> Route {
    spec.lanes()
        .into_iter()
        .find_map(|lane| spec.inline(lane).or_else(|| spec.docked(lane)))
        .unwrap_or_else(|| Route::rest(spec.output[1], Vec::new()))
}

#[cfg(test)]
#[path = "route_tests.rs"]
mod tests;
