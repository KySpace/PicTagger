function axisRange(axisType, limits, minIndex, maxIndex) {
  if (!limits) {
    return undefined;
  }

  const min = limits[minIndex];
  const max = limits[maxIndex];
  if (axisType === "log") {
    if (min <= 0 || max <= 0) {
      return undefined;
    }
    return [Math.log10(min), Math.log10(max)];
  }
  return [min, max];
}

function markerOpacity(points) {
  if (points.length === 0) {
    return [];
  }

  const weights = points.map((point) => point.weight);
  const min = Math.min(...weights);
  const max = Math.max(...weights);
  if (min === max) {
    return points.map(() => 0.85);
  }

  return weights.map((weight) => 0.25 + ((weight - min) / (max - min)) * 0.75);
}

export function renderPlotlyScatter(
  element,
  payloadJson,
  onSelect,
  onHover,
  onUnhover,
) {
  if (!globalThis.Plotly) {
    element.innerHTML =
      '<div class="plotly-missing">Plotly failed to load. Check network access for the Plotly script.</div>';
    return;
  }

  const payload = JSON.parse(payloadJson);
  const points = payload.points ?? [];
  const selectedId = payload.selected_id;
  const threshold = Number(payload.weight_threshold ?? 0);
  const customData = points.map((point) => [point.id, point.pair_index]);
  const isBelowThreshold = (point) =>
    payload.threshold_mode && point.weight < threshold;
  const lineWidths = points.map((point) => {
    if (point.id === selectedId) {
      return 2.5;
    }
    return isBelowThreshold(point) ? 1 : 0;
  });
  const lineColors = points.map((point) => {
    if (point.id === selectedId) {
      return "#c66a00";
    }
    return isBelowThreshold(point) ? point.color : "transparent";
  });

  element.__pictaggerOnSelect = onSelect;
  element.__pictaggerOnHover = onHover;
  element.__pictaggerOnUnhover = onUnhover;

  const trace = {
    type: "scatter",
    mode: "markers",
    x: points.map((point) => point.ib),
    y: points.map((point) => point.frequency),
    customdata: customData,
    text: points.map(
      (point) =>
        `source: ${point.source}<br>` +
        `source_tag: ${point.source_tag}<br>` +
        `tag: ${point.tag || "No tag"}<br>` +
        `pair: ${point.pair_index + 1}<br>` +
        `IB: ${point.ib}<br>` +
        `frequency: ${point.frequency}<br>` +
        `weight: ${point.weight}`,
    ),
    hovertemplate: "%{text}<extra></extra>",
    marker: {
      size: 9,
      color: points.map((point) =>
        isBelowThreshold(point) ? "rgba(255,255,255,0)" : point.color,
      ),
      opacity: payload.threshold_mode ? 1 : markerOpacity(points),
      line: {
        color: lineColors,
        width: lineWidths,
      },
    },
  };

  const layout = {
    autosize: true,
    margin: { l: 58, r: 18, t: 12, b: 48 },
    dragmode: "pan",
    hovermode: "closest",
    paper_bgcolor: "rgba(0,0,0,0)",
    plot_bgcolor: "#f9fcf8",
    xaxis: {
      title: { text: "IB" },
      type: payload.x_axis_type,
      range: axisRange(payload.x_axis_type, payload.manual_limits, 0, 1),
      zeroline: false,
      gridcolor: "#d6ddd2",
      linecolor: "#68726f",
      mirror: true,
    },
    yaxis: {
      title: { text: "frequency" },
      type: payload.y_axis_type,
      range: axisRange(payload.y_axis_type, payload.manual_limits, 2, 3),
      zeroline: false,
      gridcolor: "#d6ddd2",
      linecolor: "#68726f",
      mirror: true,
    },
    showlegend: false,
  };

  const config = {
    responsive: true,
    scrollZoom: true,
    displaylogo: false,
    modeBarButtonsToRemove: ["lasso2d", "select2d"],
  };

  globalThis.Plotly.react(element, [trace], layout, config).then(() => {
    if (element.__pictaggerPlotlyHandlers || typeof element.on !== "function") {
      return;
    }

    element.on("plotly_click", (event) => {
      const id = event?.points?.[0]?.customdata?.[0];
      if (id) {
        element.__pictaggerOnSelect(id);
      }
    });
    element.on("plotly_hover", (event) => {
      const id = event?.points?.[0]?.customdata?.[0];
      const pairIndex = event?.points?.[0]?.customdata?.[1];
      if (id) {
        element.__pictaggerOnHover(`${id}:${pairIndex ?? 0}`);
      }
    });
    element.on("plotly_unhover", () => {
      element.__pictaggerOnUnhover();
    });
    element.__pictaggerPlotlyHandlers = true;
  });
}
